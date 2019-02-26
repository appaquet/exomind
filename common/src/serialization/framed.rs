//!
//! This module defines a framing structure for Capn Proto messages, allowing us to encode
//! message type, message size, message data and a signature of the actual message data.
//!
//! A frame is structured like this:
//! ---------------------------------------------------------
//! | Metadata | Message           | Signature    | Metadata|
//! ---------------------------------------------------------
//!
//! Metadata at the head and tail of the frame are the same, are repeated to support
//! forward and backward iterations.
//!

use std;
use std::sync::Once;

use byteorder;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use capnp;
pub use capnp::message::ReaderSegments;
use capnp::message::{Allocator, Builder, HeapAllocator, Reader};
use capnp::serialize::SliceSegments;
use lazycell::AtomicLazyCell;
use owning_ref::OwningHandle;

use crate::security::hash::Multihash;

///
/// Trait that needs to have an impl for each capnp generated message struct.
/// Used to identify a unique type id for each message and annotate each framed message.
///
pub trait MessageType<'a>: capnp::traits::Owned<'a> {
    fn message_type() -> u16;
}

///
/// A Framed Message is a capnp message with an extra header identifying the message type and message size.
///
pub trait Frame: SignedFrame {
    fn message_type(&self) -> u16;
    fn message_size(&self) -> usize;
    fn frame_size(&self) -> usize;
    fn get_typed_reader<'b, T: MessageType<'b>>(
        &'b self,
    ) -> Result<<T as capnp::traits::Owned>::Reader, Error>;
    fn copy_into(&self, buf: &mut [u8]);
    fn to_owned(&self) -> OwnedFrame;
}

///
/// A Framed Typed Message wraps a FramedMessage annotated type.
///
pub trait TypedFrame<T>: SignedFrame
where
    T: for<'a> MessageType<'a>,
{
    fn message_type(&self) -> u16;
    fn message_size(&self) -> usize;
    fn frame_size(&self) -> usize;
    fn get_typed_reader(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error>;
    fn copy_into(&self, buf: &mut [u8]);
    fn to_owned(&self) -> OwnedTypedFrame<T>;
}

///
///
///
pub trait SignedFrame {
    fn message_data(&self) -> &[u8];
    fn signature_data(&self) -> Option<&[u8]>;
}

///
/// Framed message builder
///
pub struct FrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    message_type: u16,
    builder: Builder<HeapAllocator>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> FrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn new() -> FrameBuilder<T> {
        let message_type = <T as MessageType>::message_type();
        let mut builder = Builder::new_default();
        builder.init_root::<<T as capnp::traits::Owned>::Builder>();
        FrameBuilder {
            message_type,
            builder,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn get_builder(&mut self) -> &mut Builder<HeapAllocator> {
        &mut self.builder
    }

    pub fn get_builder_typed(&mut self) -> <T as capnp::traits::Owned>::Builder {
        self.builder.get_root().unwrap()
    }

    pub fn as_owned_framed<S: FrameSigner>(&self, signer: S) -> Result<OwnedTypedFrame<T>, Error> {
        let msg = OwnedFrame::from_builder(self.message_type, &self.builder, signer)?;
        Ok(msg.into_typed())
    }

    pub fn as_owned_unsigned_framed(&self) -> Result<OwnedTypedFrame<T>, Error> {
        self.as_owned_framed(NullFrameSigner)
    }

    pub fn into_framed_vec<S: FrameSigner>(self, signer: S) -> Result<Vec<u8>, Error> {
        let mut writer = OwnedFrameWriter::new(self.message_type, signer);
        capnp::serialize::write_message(&mut writer, &self.builder)?;
        let (buffer, _metadata) = writer.finish()?;
        Ok(buffer)
    }

    pub fn into_unsigned_framed_bytes(self) -> Result<Vec<u8>, Error> {
        self.into_framed_vec(NullFrameSigner)
    }

    pub fn write_into<S: FrameSigner>(
        &self,
        data: &mut [u8],
        signer: S,
    ) -> Result<FrameMetadata, Error> {
        write_framed_builder_into_buffer(data, self.message_type, &self.builder, signer)
    }

    pub fn write_into_unsigned(&self, data: &mut [u8]) -> Result<FrameMetadata, Error> {
        write_framed_builder_into_buffer(data, self.message_type, &self.builder, NullFrameSigner)
    }
}

impl<T> Default for FrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    fn default() -> Self {
        FrameBuilder::new()
    }
}

///
/// Framed message coming from a slice of bytes. No copy was involved to create this message, as it uses the underlying bytes slice.
///
/// Message parsing into an actual capnp message is lazily done when `get_typed_reader()` is called.
///
pub struct SliceFrame<'a> {
    metadata: FrameMetadata,
    data: &'a [u8],
    lazy_reader: AtomicLazyCell<Result<Reader<SliceSegments<'a>>, Error>>,
    lazy_reader_once: Once,
}

impl<'a> SliceFrame<'a> {
    pub fn new(data: &[u8]) -> Result<SliceFrame, Error> {
        let header_metadata = FrameMetadata::from_slice(data)?;

        if header_metadata.message_size == 0 {
            error!("Message from slice had an size of 0");
            return Err(Error::EOF);
        }

        if data.len() < header_metadata.frame_size() {
            error!(
                "Slice size is smaller than expected frame size. Slice size {} < Expected size {}",
                data.len(),
                header_metadata.frame_size()
            );
            return Err(Error::InvalidSize);
        }

        let footer_metadata = FrameMetadata::from_slice(&data[header_metadata.footer_offset()..])?;
        if header_metadata != footer_metadata {
            error!(
                "Frame's header metadata is not the same as the footer's metadata: {:?} != {:?}",
                header_metadata, footer_metadata
            );
            return Err(Error::InvalidData);
        }

        Ok(SliceFrame {
            metadata: header_metadata,
            data,
            lazy_reader: AtomicLazyCell::new(),
            lazy_reader_once: Once::new(),
        })
    }

    pub fn new_from_next_offset(data: &[u8], next_offset: usize) -> Result<SliceFrame, Error> {
        if data.len() < next_offset || next_offset < FrameMetadata::SIZE {
            error!(
                "Tried to read from next offset {} in buffer of len {}",
                next_offset,
                data.len()
            );
            return Err(Error::InvalidSize);
        }

        let footer_metadata_offset = next_offset - FrameMetadata::SIZE;
        let footer_metadata = FrameMetadata::from_slice(&data[footer_metadata_offset..])?;
        if footer_metadata.frame_size() > next_offset {
            error!(
                "End frame size would exceed buffer 0th position (frame_size={} > next_offset={})",
                footer_metadata.frame_size(),
                next_offset
            );
            return Err(Error::InvalidSize);
        }

        let frame_begin = next_offset - footer_metadata.frame_size();
        let header_metadata = FrameMetadata::from_slice(&data[frame_begin..])?;
        if header_metadata != footer_metadata {
            error!(
                "Frame's header metadata is not the same as the footer's metadata: {:?} != {:?}",
                header_metadata, footer_metadata
            );
            return Err(Error::InvalidData);
        }

        if header_metadata.message_size == 0 {
            error!("Message from slice had an size of 0");
            return Err(Error::EOF);
        }

        Ok(SliceFrame {
            metadata: header_metadata,
            data: &data[frame_begin..frame_begin + footer_metadata.frame_size()],
            lazy_reader: AtomicLazyCell::new(),
            lazy_reader_once: Once::new(),
        })
    }

    fn read_capn_message(buffer: &[u8]) -> Result<Reader<SliceSegments>, Error> {
        let words = unsafe { capnp::Word::bytes_to_words(buffer) };
        let opts = capnp::message::ReaderOptions::new();
        capnp::serialize::read_message_from_words(&words, opts).map_err(|err| {
            warn!("Couldn't deserialize message reader: {:?}", err);
            Error::InvalidData
        })
    }

    pub fn to_owned(&self) -> OwnedFrame {
        OwnedFrame::new(self.data.to_vec()).unwrap()
    }

    pub fn into_typed<T>(self) -> TypedSliceFrame<'a, T>
    where
        T: MessageType<'a>,
    {
        TypedSliceFrame {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Frame for SliceFrame<'a> {
    fn message_type(&self) -> u16 {
        self.metadata.message_type
    }

    fn message_size(&self) -> usize {
        self.metadata.message_size
    }

    fn frame_size(&self) -> usize {
        self.metadata.frame_size()
    }

    fn get_typed_reader<'b, T: MessageType<'b>>(
        &'b self,
    ) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        // Unfortunately, the LazyCell is nice to use when single thread, but in multi-thread,
        // it doesn't have a "borrow_with" method that initializes if not already initialized.
        // We use a Once to make sure we initialize the reader if needed.
        if !self.lazy_reader.filled() {
            self.lazy_reader_once.call_once(|| {
                let message_range = self.metadata.message_range();
                self.lazy_reader
                    .fill(Self::read_capn_message(&self.data[message_range]))
                    .map_err(|_| ())
                    .expect("Lazy ready was already initialized, which should be impossible since it's inside a Once");
            });
        }

        let reader = self
            .lazy_reader
            .borrow()
            .expect("Tried to unwrap the lazy reader that should have been filled");

        let reader = reader.as_ref().map_err(|err| {
            // needed since the cell contains a ref to the error, and we cannot return it directly
            err.clone()
        })?;

        reader.get_root().map_err(|_err| Error::InvalidData)
    }

    fn copy_into(&self, buf: &mut [u8]) {
        buf[0..self.metadata.frame_size()].copy_from_slice(self.data);
    }

    fn to_owned(&self) -> OwnedFrame {
        OwnedFrame::new(self.data.to_vec())
            .expect("Couldn't create owned message from slice, which shouldn't be possible")
    }
}

impl<'a> SignedFrame for SliceFrame<'a> {
    fn message_data(&self) -> &[u8] {
        &self.data[self.metadata.message_range()]
    }

    fn signature_data(&self) -> Option<&[u8]> {
        self.metadata.signature_range().map(|r| &self.data[r])
    }
}

///
/// A framed typed message coming from a slice of bytes that wraps a `SliceFrame` with annotated type.
///
pub struct TypedSliceFrame<'a, T>
where
    T: MessageType<'a>,
{
    message: SliceFrame<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> TypedSliceFrame<'a, T>
where
    T: MessageType<'a>,
{
    pub fn new(data: &'a [u8]) -> Result<TypedSliceFrame<'a, T>, Error> {
        let expected_type = <T as MessageType>::message_type();
        let message = SliceFrame::new(data)?;
        if message.message_type() != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type,
                message.message_type()
            );
            return Err(Error::InvalidData);
        }

        Ok(TypedSliceFrame {
            message,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_from_next_offset(
        data: &'a [u8],
        next_offset: usize,
    ) -> Result<TypedSliceFrame<'a, T>, Error> {
        let expected_type = <T as MessageType>::message_type();
        let message = SliceFrame::new_from_next_offset(data, next_offset)?;
        if message.message_type() != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type,
                message.message_type()
            );
            return Err(Error::InvalidData);
        }

        Ok(TypedSliceFrame {
            message,
            phantom: std::marker::PhantomData,
        })
    }
}

impl<'a, T> TypedFrame<T> for TypedSliceFrame<'a, T>
where
    T: for<'b> MessageType<'b>,
{
    fn message_type(&self) -> u16 {
        self.message.message_type()
    }

    fn message_size(&self) -> usize {
        self.message.message_size()
    }

    fn frame_size(&self) -> usize {
        self.message.frame_size()
    }

    fn get_typed_reader(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        self.message.get_typed_reader::<T>()
    }

    fn copy_into(&self, buf: &mut [u8]) {
        self.message.copy_into(buf);
    }

    fn to_owned(&self) -> OwnedTypedFrame<T> {
        let owned_message = self.message.to_owned();
        owned_message.into_typed()
    }
}

impl<'a, T> SignedFrame for TypedSliceFrame<'a, T>
where
    T: for<'b> MessageType<'b>,
{
    fn message_data(&self) -> &[u8] {
        self.message.message_data()
    }

    fn signature_data(&self) -> Option<&[u8]> {
        self.message.signature_data()
    }
}

///
/// Iterator on a stream of untyped framed messages.
/// Will return None on error, and the `last_error` field will identify the error, if any.
///
pub struct FramesIterator<'a> {
    buffer: &'a [u8],
    current_offset: usize,
    pub last_error: Option<Error>,
}

impl<'a> FramesIterator<'a> {
    pub fn new(buffer: &'a [u8]) -> FramesIterator<'a> {
        FramesIterator {
            buffer,
            current_offset: 0,
            last_error: None,
        }
    }
}

impl<'a> Iterator for FramesIterator<'a> {
    type Item = IteratedFrame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset;
        let slice = &self.buffer[offset..];
        match SliceFrame::new(slice) {
            Ok(framed_message) => {
                self.current_offset += framed_message.frame_size();
                Some(IteratedFrame {
                    offset,
                    framed_message,
                })
            }
            Err(Error::EOF) => {
                trace!("Reached EOF");
                None
            }
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }
}

pub struct IteratedFrame<'a> {
    pub offset: usize,
    pub framed_message: SliceFrame<'a>,
}

///
/// A standalone framed message.
///
/// Uses a OwningHandle in order to prevent having data twice in memory and use a FramedSliceMessage
/// that references data that is stored in struct itself.
///
/// See https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct
/// As noted here: https://github.com/Kimundi/owning-ref-rs/issues/27
/// We should never expose the 'static lifetime through the API because it may lead into unsafe
/// behaviour.
///
pub struct OwnedFrame {
    owned_slice_message: OwningHandle<Vec<u8>, Box<SliceFrame<'static>>>,
}

impl OwnedFrame {
    pub fn new(data: Vec<u8>) -> Result<OwnedFrame, Error> {
        let owned_slice_message = OwningHandle::try_new(data, |data| unsafe {
            SliceFrame::new(data.as_ref().unwrap()).map(Box::new)
        })?;

        Ok(OwnedFrame {
            owned_slice_message,
        })
    }

    pub fn from_builder<A: Allocator, S: FrameSigner>(
        message_type: u16,
        builder: &Builder<A>,
        signer: S,
    ) -> Result<OwnedFrame, Error> {
        let mut writer = OwnedFrameWriter::new(message_type, signer);
        capnp::serialize::write_message(&mut writer, builder).unwrap();
        let (buffer, _metadata) = writer.finish()?;

        OwnedFrame::new(buffer)
    }

    pub fn into_typed<T>(self) -> OwnedTypedFrame<T>
    where
        T: for<'a> MessageType<'a>,
    {
        OwnedTypedFrame {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl Clone for OwnedFrame {
    fn clone(&self) -> Self {
        OwnedFrame::new(self.owned_slice_message.data.to_vec()).unwrap()
    }
}

impl Frame for OwnedFrame {
    fn message_type(&self) -> u16 {
        self.owned_slice_message.message_type()
    }

    fn message_size(&self) -> usize {
        self.owned_slice_message.message_size()
    }

    fn frame_size(&self) -> usize {
        self.owned_slice_message.frame_size()
    }

    fn get_typed_reader<'b, T: MessageType<'b>>(
        &'b self,
    ) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        self.owned_slice_message.get_typed_reader::<T>()
    }

    fn copy_into(&self, buf: &mut [u8]) {
        self.owned_slice_message.copy_into(buf)
    }

    fn to_owned(&self) -> OwnedFrame {
        OwnedFrame::new(self.owned_slice_message.data.to_vec()).unwrap()
    }
}

impl SignedFrame for OwnedFrame {
    fn message_data(&self) -> &[u8] {
        self.owned_slice_message.message_data()
    }

    fn signature_data(&self) -> Option<&[u8]> {
        self.owned_slice_message.signature_data()
    }
}

///
/// A standalone framed typed message that wraps a `OwnedFrame` with annotated type.
///
#[derive(Clone)]
pub struct OwnedTypedFrame<T>
where
    T: for<'a> MessageType<'a>,
{
    message: OwnedFrame,
    phantom: std::marker::PhantomData<T>,
}

impl<T> TypedFrame<T> for OwnedTypedFrame<T>
where
    T: for<'a> MessageType<'a>,
{
    fn message_type(&self) -> u16 {
        self.message.message_type()
    }

    fn message_size(&self) -> usize {
        self.message.message_size()
    }

    fn frame_size(&self) -> usize {
        self.message.frame_size()
    }

    fn get_typed_reader(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        self.message.get_typed_reader::<T>()
    }

    fn copy_into(&self, buf: &mut [u8]) {
        self.message.copy_into(buf);
    }

    fn to_owned(&self) -> OwnedTypedFrame<T> {
        let owned_message = self.message.clone();
        owned_message.into_typed()
    }
}

impl<T> SignedFrame for OwnedTypedFrame<T>
where
    T: for<'b> MessageType<'b>,
{
    fn message_data(&self) -> &[u8] {
        self.message.message_data()
    }

    fn signature_data(&self) -> Option<&[u8]> {
        self.message.signature_data()
    }
}

///
/// Framed message writer that wraps a slice, that should have enough capacity, and exposes a Write implementation used by capnp
///
struct SliceFrameWriter<'a, S: FrameSigner> {
    message_type: u16,
    buffer: &'a mut [u8],
    count: usize,
    signer: S,
}

impl<'a, S: FrameSigner> SliceFrameWriter<'a, S> {
    fn new(message_type: u16, buffer: &'a mut [u8], signer: S) -> SliceFrameWriter<'a, S> {
        SliceFrameWriter {
            message_type,
            buffer,
            count: FrameMetadata::SIZE,
            signer,
        }
    }

    fn finish(self) -> Result<FrameMetadata, Error> {
        let SliceFrameWriter {
            message_type,
            mut buffer,
            count,
            signer,
        } = self;

        let message_size = count - FrameMetadata::SIZE;

        // write signature
        let signature_size = match signer.finish() {
            Some(signature) => Self::checked_copy_to_buffer(count, &mut buffer, &signature)?,
            None => 0,
        };

        // write metadata at beginning and end of the buffer
        let metadata = FrameMetadata {
            message_type,
            message_size,
            signature_size,
        };
        metadata.copy_into_slice(&mut buffer[0..])?;
        metadata.copy_into_slice(&mut buffer[metadata.footer_offset()..])?;

        Ok(metadata)
    }

    fn checked_copy_to_buffer(
        offset: usize,
        buffer: &mut [u8],
        data: &[u8],
    ) -> Result<usize, std::io::Error> {
        let offset_from = offset;
        let len = data.len();
        let offset_to = offset_from + len;

        if offset_to > buffer.len() {
            error!(
                "Tried to write a message that exceeded size of buffer: offset_to={} buffer_len={}",
                offset_to,
                buffer.len()
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Message bigger than buffer len",
            ));
        }

        buffer[offset_from..offset_to].copy_from_slice(data);

        Ok(data.len())
    }
}

impl<'a, S: FrameSigner> std::io::Write for SliceFrameWriter<'a, S> {
    fn write(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        let written = Self::checked_copy_to_buffer(self.count, &mut self.buffer, data)?;
        self.count += written;
        self.signer.consume(data);
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

///
/// Helper method that writes a single message into a buffer. Uses a `FramedMessageWriter`
///
pub fn write_framed_builder_into_buffer<A: capnp::message::Allocator, S: FrameSigner>(
    buffer: &mut [u8],
    message_type: u16,
    message_builder: &capnp::message::Builder<A>,
    signer: S,
) -> Result<FrameMetadata, Error> {
    let mut framed_writer = SliceFrameWriter::new(message_type, buffer, signer);
    capnp::serialize::write_message(&mut framed_writer, &message_builder)?;
    framed_writer.finish()
}

///
/// Framed message writer that writes into an owned Vector (and resizes itself), and exposes a Write implementation used by capnp
///
pub struct OwnedFrameWriter<S: FrameSigner> {
    message_type: u16,
    buffer: Vec<u8>,
    signer: S,
}

impl<S: FrameSigner> OwnedFrameWriter<S> {
    pub fn new(message_type: u16, signer: S) -> OwnedFrameWriter<S> {
        let mut buffer = Vec::new();
        Self::push_empty_bytes(&mut buffer, FrameMetadata::SIZE);
        OwnedFrameWriter {
            message_type,
            buffer,
            signer,
        }
    }

    pub fn finish(mut self) -> Result<(Vec<u8>, FrameMetadata), Error> {
        let message_size = self.buffer.len() - FrameMetadata::SIZE;

        // write signature
        let signature_size = match self.signer.finish() {
            Some(signature) => {
                let sig_size = signature.len();
                for elem in signature {
                    self.buffer.push(elem);
                }
                sig_size
            }
            None => 0,
        };

        let metadata = FrameMetadata {
            message_type: self.message_type,
            message_size,
            signature_size,
        };

        // copy metadata at beginning
        metadata.copy_into_slice(&mut self.buffer[0..])?;

        // copy metadata at end
        Self::push_empty_bytes(&mut self.buffer, FrameMetadata::SIZE);
        metadata.copy_into_slice(&mut self.buffer[metadata.footer_offset()..])?;

        Ok((self.buffer, metadata))
    }

    fn push_empty_bytes(buffer: &mut Vec<u8>, count: usize) {
        for _i in 0..count {
            buffer.push(0);
        }
    }
}

impl<'a, S: FrameSigner> std::io::Write for OwnedFrameWriter<S> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        for elem in buf {
            self.buffer.push(*elem);
        }
        self.signer.consume(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

///
/// Metadata written at beginning and at the end of a frame.
///
#[derive(Debug, PartialEq)]
pub struct FrameMetadata {
    pub message_type: u16,
    pub message_size: usize,
    pub signature_size: usize,
}

impl FrameMetadata {
    const TYPE_FIELD_SIZE: usize = 2;
    const DATA_FIELD_SIZE: usize = 4;
    const SIG_FIELD_SIZE: usize = 2;
    const SIZE: usize = Self::TYPE_FIELD_SIZE + Self::DATA_FIELD_SIZE + Self::SIG_FIELD_SIZE;

    fn from_slice(buffer: &[u8]) -> Result<FrameMetadata, Error> {
        if buffer.len() < Self::SIZE {
            return Err(Error::EOF);
        }

        let message_type = LittleEndian::read_u16(&buffer[0..Self::TYPE_FIELD_SIZE]);
        let message_size = LittleEndian::read_u32(&buffer[2..2 + Self::DATA_FIELD_SIZE]) as usize;
        let signature_size =
            usize::from(LittleEndian::read_u16(&buffer[6..6 + Self::SIG_FIELD_SIZE]));

        Ok(FrameMetadata {
            message_type,
            message_size,
            signature_size,
        })
    }

    fn copy_into_slice(&self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() < Self::SIZE {
            return Err(Error::DestinationSize);
        }

        LittleEndian::write_u16(&mut buffer[0..Self::TYPE_FIELD_SIZE], self.message_type);
        LittleEndian::write_u32(
            &mut buffer[2..2 + Self::DATA_FIELD_SIZE],
            self.message_size as u32,
        );
        LittleEndian::write_u16(
            &mut buffer[6..6 + Self::SIG_FIELD_SIZE],
            self.signature_size as u16,
        );

        Ok(())
    }

    #[inline]
    fn frame_size(&self) -> usize {
        Self::SIZE + Self::SIZE + self.message_size + self.signature_size
    }

    #[inline]
    fn message_offset(&self) -> usize {
        Self::SIZE
    }

    #[inline]
    fn footer_offset(&self) -> usize {
        Self::SIZE + self.message_size + self.signature_size
    }

    #[inline]
    fn message_range(&self) -> std::ops::Range<usize> {
        let message_offset = self.message_offset();
        message_offset..message_offset + self.message_size
    }

    #[inline]
    fn has_signature(&self) -> bool {
        self.signature_size > 0
    }

    #[allow(dead_code)]
    #[inline]
    fn signature_range(&self) -> Option<std::ops::Range<usize>> {
        if self.has_signature() {
            let signature_offset = self.message_offset() + self.message_size;
            Some(signature_offset..signature_offset + self.signature_size)
        } else {
            None
        }
    }
}

///
/// Trait representing a way to sign a frame
///
pub trait FrameSigner {
    fn consume(&mut self, data: &[u8]);
    fn finish(self) -> Option<Vec<u8>>;
}

pub struct NullFrameSigner;

impl FrameSigner for NullFrameSigner {
    fn consume(&mut self, _data: &[u8]) {}

    fn finish(self) -> Option<Vec<u8>> {
        None
    }
}

pub struct MultihashFrameSigner<H>
where
    H: crate::security::hash::StreamHasher,
{
    hasher: H,
}

impl MultihashFrameSigner<crate::security::hash::Sha3Hasher> {
    pub fn new_sha3256() -> MultihashFrameSigner<crate::security::hash::Sha3Hasher> {
        MultihashFrameSigner {
            hasher: crate::security::hash::Sha3Hasher::new_256(),
        }
    }

    pub fn new_sha3512() -> MultihashFrameSigner<crate::security::hash::Sha3Hasher> {
        MultihashFrameSigner {
            hasher: crate::security::hash::Sha3Hasher::new_512(),
        }
    }

    pub fn validate<S: SignedFrame>(frame: &S) -> Result<Multihash, Error> {
        frame
            .signature_data()
            .ok_or(Error::InvalidSignature)
            .and_then(|data| {
                let hash_msg =
                    Multihash::from_bytes(data.to_vec()).map_err(|_err| Error::InvalidSignature)?;
                let hash_data = crate::security::hash::multihash::encode(
                    hash_msg.algorithm(),
                    frame.message_data(),
                )
                .map_err(|_err| Error::InvalidSignature)?;

                if hash_data != hash_msg {
                    return Err(Error::InvalidSignature);
                }

                Ok(hash_data)
            })
    }
}

impl<H> MultihashFrameSigner<H>
where
    H: crate::security::hash::StreamHasher,
{
    pub fn new(hasher: H) -> MultihashFrameSigner<H> {
        MultihashFrameSigner { hasher }
    }
}

impl<H> FrameSigner for MultihashFrameSigner<H>
where
    H: crate::security::hash::StreamHasher,
{
    fn consume(&mut self, data: &[u8]) {
        self.hasher.consume(data);
    }

    fn finish(self) -> Option<Vec<u8>> {
        Some(self.hasher.into_mulithash_bytes())
    }
}

///
/// Framing error
///
#[derive(Fail, Debug, Clone, PartialEq)]
#[fail(display = "A message serialization error occurred")]
pub enum Error {
    #[fail(display = "Couldn't deserialization data")]
    InvalidData,
    #[fail(display = "Invalid message size")]
    InvalidSize,
    #[fail(display = "Destination size is too small")]
    DestinationSize,
    #[fail(display = "Signature validation error")]
    InvalidSignature,
    #[fail(display = "Capnp serialization error")]
    CapnpSerialization(capnp::ErrorKind),
    #[fail(display = "IO error")]
    IO(std::io::ErrorKind),
    #[fail(display = "Reached end of message / stream")]
    EOF,
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::CapnpSerialization(err.kind)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.kind())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::data_chain_capnp::block;

    use super::*;

    #[test]
    fn write_and_read_frame_metadata() {
        let mut buf = vec![0; 10];

        let meta1 = FrameMetadata {
            message_type: 3,
            message_size: 5,
            signature_size: 7,
        };
        meta1.copy_into_slice(&mut buf).unwrap();

        let meta2 = FrameMetadata::from_slice(&buf).unwrap();
        assert_eq!(meta1, meta2);
    }

    #[test]
    fn frame_builder_into_bytes() {
        let block_builder = build_test_block(123, 321);

        let frame_data = block_builder.into_unsigned_framed_bytes().unwrap();
        let frame_slice = SliceFrame::new(&frame_data).unwrap();
        assert_eq!(
            frame_slice.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        let typed_frame = frame_slice.into_typed::<block::Owned>();
        let block_reader = typed_frame.get_typed_reader().unwrap();
        assert_eq!(block_reader.get_offset(), 123);
        assert_eq!(typed_frame.frame_size(), frame_data.len());
    }

    #[test]
    fn frame_builder_into_owned_frame() {
        let block_builder = build_test_block(123, 10000);

        let owned_frame = block_builder.as_owned_unsigned_framed().unwrap();
        assert_eq!(
            owned_frame.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        let block_reader = owned_frame.get_typed_reader().unwrap();
        assert_eq!(block_reader.get_offset(), 123);
    }

    #[test]
    fn frame_builder_write_into_buffer() {
        let block_builder = build_test_block(0, 10000);

        let mut data = [0u8; 1000];
        let frame_metadata = block_builder.write_into_unsigned(&mut data).unwrap();

        let framed_data = block_builder.into_unsigned_framed_bytes().unwrap();
        assert_eq!(&framed_data[..], &data[..frame_metadata.frame_size()]);
    }

    #[test]
    fn slice_frame_from_invalid_data() {
        // no data found
        let data = [0u8; 1000];
        assert_eq!(SliceFrame::new(&data).err(), Some(Error::EOF));

        // invalid size
        let mut data = [0u8; 1000];
        LittleEndian::write_u16(&mut data, 10);
        LittleEndian::write_u32(&mut data, 10);
        assert!(SliceFrame::new(&data).is_err());

        // overflow size
        let mut data = [0u8; 1000];
        LittleEndian::write_u16(&mut data, 10);
        LittleEndian::write_u32(&mut data, 10000);
        assert!(SliceFrame::new(&data).is_err());
    }

    #[test]
    fn slice_frame_from_next_offset() -> Result<(), Error> {
        let mut data = [0u8; 1000];

        let mut block1_builder = build_test_block(0, 10000);
        let block1_metadata = write_framed_builder_into_buffer(
            &mut data[0..],
            123,
            &block1_builder.get_builder(),
            NullFrameSigner,
        )?;

        let block2_offset = block1_metadata.frame_size();
        let mut block2_builder = build_test_block(1, 10001);
        let block2_metadata = write_framed_builder_into_buffer(
            &mut data[block2_offset..],
            456,
            &block2_builder.get_builder(),
            NullFrameSigner,
        )?;

        let block3_offset = block2_offset + block2_metadata.frame_size();
        let mut block3_builder = build_test_block(2, 10002);
        let block3_metadata = write_framed_builder_into_buffer(
            &mut data[block3_offset..],
            789,
            &block3_builder.get_builder(),
            NullFrameSigner,
        )?;

        dbg!(block3_offset);

        // wrong offset tests
        assert!(SliceFrame::new_from_next_offset(&data[0..], 0).is_err());
        assert!(SliceFrame::new_from_next_offset(&data[0..], 112).is_err());

        let frame = SliceFrame::new_from_next_offset(&data[0..], block1_metadata.frame_size())?;
        assert_eq!(frame.message_type(), 123);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_offset(), 0);

        let frame = SliceFrame::new_from_next_offset(&data[0..], block3_offset)?;
        assert_eq!(frame.message_type(), 456);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_offset(), 1);

        let frame = SliceFrame::new_from_next_offset(
            &data[0..],
            block3_offset + block3_metadata.frame_size(),
        )?;
        assert_eq!(frame.message_type(), 789);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_offset(), 2);

        Ok(())
    }

    #[test]
    fn frame_write_fail_if_not_enough_space() {
        let mut block_builder = build_test_block(0, 10000);

        let mut data = [0u8; 10];
        assert!(write_framed_builder_into_buffer(
            &mut data,
            123,
            block_builder.get_builder(),
            NullFrameSigner,
        )
        .is_err());
    }

    #[test]
    fn frames_iterator() {
        let mut data = [0u8; 500_000];

        let mut next_offset = 0;
        for i in 0..1000 {
            let mut block_builder = build_test_block(i as u64, (i * 10000) as u64);

            let metadata = write_framed_builder_into_buffer(
                &mut data[next_offset..],
                123,
                block_builder.get_builder(),
                NullFrameSigner,
            )
            .unwrap();
            next_offset += metadata.frame_size();
        }

        // simple forward iteration
        let mut iterator = FramesIterator::new(&data);
        let mut last_offset = 0;
        for frame in iterator.by_ref() {
            assert!(last_offset == 0 || frame.offset > last_offset);
            last_offset = frame.offset;
        }
        assert_eq!(iterator.last_error, None);

        // make sure we can deserialize
        let last_iter_frame = FramesIterator::new(&data).take(1).last().unwrap();
        assert_eq!(last_iter_frame.offset, 0);

        let typed_frame = last_iter_frame.framed_message.into_typed::<block::Owned>();
        let block_reader = typed_frame.get_typed_reader().unwrap();
        assert_eq!(block_reader.get_offset(), 0);

        // iterator typing
        let frames_iter = FramesIterator::new(&data).take(10);
        let hashes: Vec<u64> = frames_iter
            .filter(|m| m.framed_message.message_type() == 123)
            .map(|m| m.framed_message.into_typed::<block::Owned>())
            .map(|b| b.get_typed_reader().unwrap().get_offset())
            .collect();
        assert_eq!(hashes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn frames_iterator_error_handling() {
        // invalid data should have an error
        let mut data = [0u8; 1000];
        LittleEndian::write_u16(&mut data, 10);
        LittleEndian::write_u32(&mut data[2..], 42);

        let mut iterator = FramesIterator::new(&data);
        assert_eq!(iterator.by_ref().count(), 0);
        assert!(iterator.last_error.is_some());
    }

    #[test]
    fn frame_sign_and_validate() {
        let block_builder = build_test_block(0, 10000);

        let signer = MultihashFrameSigner::new_sha3256();
        let frame: OwnedTypedFrame<block::Owned> = block_builder.as_owned_framed(signer).unwrap();
        assert_eq!(frame.signature_data().unwrap().len(), 2 + 32);
        assert!(MultihashFrameSigner::validate(&frame).is_ok());

        let signer = MultihashFrameSigner::new_sha3512();
        let frame = block_builder.as_owned_framed(signer).unwrap();
        assert_eq!(frame.signature_data().unwrap().len(), 2 + 64);
        assert!(MultihashFrameSigner::validate(&frame).is_ok());

        let mut data = block_builder
            .into_framed_vec(MultihashFrameSigner::new_sha3512())
            .unwrap();
        let frame = SliceFrame::new(&data).unwrap();
        assert!(MultihashFrameSigner::validate(&frame).is_ok());

        // modify message should invalidate signature
        data[10] = 40;
        data[11] = 12;
        let frame = SliceFrame::new(&data).unwrap();
        assert!(MultihashFrameSigner::validate(&frame).is_err());
    }

    fn build_test_block(block_offset: u64, entry_id: u64) -> FrameBuilder<block::Owned> {
        let mut block_msg_builder = FrameBuilder::<block::Owned>::new();

        let mut block_builder = block_msg_builder.get_builder_typed();
        block_builder.set_offset(block_offset);

        let entries = block_builder.init_entries(1);
        {
            let mut entry = entries.get(0);
            entry.set_id(entry_id);
        }

        block_msg_builder
    }
}
