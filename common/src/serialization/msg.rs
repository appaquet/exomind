use std;

use byteorder;
use byteorder::ByteOrder;
use capnp;
pub use capnp::message::ReaderSegments;
use capnp::message::{Allocator, Builder, HeapAllocator, Reader};
use capnp::serialize::SliceSegments;
use lazycell::LazyCell;
use owning_ref::OwningHandle;

/// Trait that needs to have an impl for each capnp generated message struct.
/// Used to identify a unique type id for each message and annotate each framed message.
pub trait MessageType<'a>: capnp::traits::Owned<'a> {
    fn message_type() -> u16;
}

/// A Framed Message is a capnp message with an extra header identifying the message type and message size.
pub trait FramedMessage {
    fn message_type(&self) -> u16;
    fn message_size(&self) -> usize;
    fn frame_size(&self) -> usize;
    fn get_typed_reader<'b, T: MessageType<'b>>(
        &'b self,
    ) -> Result<<T as capnp::traits::Owned>::Reader, Error>;
    fn copy_into(&self, buf: &mut [u8]);
    fn to_owned(&self) -> FramedOwnedMessage;
}

/// A Framed Typed Message is a FramedMessage that is guaranteed to be implementing the annotated type.
pub trait FramedTypedMessage<T>
where
    T: for<'a> MessageType<'a>,
{
    fn message_type(&self) -> u16;
    fn message_size(&self) -> usize;
    fn frame_size(&self) -> usize;
    fn get_typed_reader(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error>;
    fn copy_into(&self, buf: &mut [u8]);
    fn to_owned(&self) -> FramedOwnedTypedMessage<T>;
}

/// Message builder
pub struct MessageBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    message_type: u16,
    builder: Builder<HeapAllocator>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> MessageBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn new() -> MessageBuilder<T> {
        let message_type = <T as MessageType>::message_type();
        let mut builder = Builder::new_default();
        builder.init_root::<<T as capnp::traits::Owned>::Builder>();
        MessageBuilder {
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

    pub fn as_owned_framed(&self) -> Result<FramedOwnedTypedMessage<T>, Error> {
        let msg = FramedOwnedMessage::from_builder(self.message_type, &self.builder)?;
        Ok(msg.into_typed())
    }

    pub fn into_framed_vec(self) -> Result<Vec<u8>, Error> {
        let mut writer = OwnedFramedMessageWriter::new(self.message_type);
        capnp::serialize::write_message(&mut writer, &self.builder).map_err(|err| {
            error!("Couldn't serialize builder into framed vector: {:?}", err);
            Error::InvalidData
        })?;
        let (buffer, _metadata) = writer.finish()?;
        Ok(buffer)
    }

    pub fn write_into(&self, data: &mut [u8]) -> Result<FrameMetadata, Error> {
        write_framed_builder_into_buffer(data, self.message_type, &self.builder).map_err(|err| {
            error!("Couldn't write builder into buffer: {:?}", err);
            Error::InvalidSize
        })
    }
}

impl<T> Default for MessageBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    fn default() -> Self {
        MessageBuilder::new()
    }
}

/// Framed message coming from a slice of bytes. No copy was involved to create this message, as it uses the underlying bytes slice.
///
/// Message parsing into an actual capnp message is lazily done when `get_typed_reader()` is called.
pub struct FramedSliceMessage<'a> {
    metadata: FrameMetadata,
    data: &'a [u8],
    lazy_reader: LazyCell<Reader<SliceSegments<'a>>>,
}

impl<'a> FramedSliceMessage<'a> {
    pub fn new(data: &[u8]) -> Result<FramedSliceMessage, Error> {
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

        Ok(FramedSliceMessage {
            metadata: header_metadata,
            data,
            lazy_reader: LazyCell::new(),
        })
    }

    pub fn new_from_next_offset(
        data: &[u8],
        next_offset: usize,
    ) -> Result<FramedSliceMessage, Error> {
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

        Ok(FramedSliceMessage {
            metadata: header_metadata,
            data: &data[frame_begin..frame_begin + footer_metadata.frame_size()],
            lazy_reader: LazyCell::new(),
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

    pub fn to_owned(&self) -> FramedOwnedMessage {
        FramedOwnedMessage::new(self.data.to_vec()).unwrap()
    }

    pub fn into_typed<T>(self) -> FramedSliceTypedMessage<'a, T>
    where
        T: MessageType<'a>,
    {
        FramedSliceTypedMessage {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> FramedMessage for FramedSliceMessage<'a> {
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
        let reader = self.lazy_reader.try_borrow_with(|| {
            let message_range = self.metadata.message_range();
            Self::read_capn_message(&self.data[message_range])
        })?;

        reader.get_root().map_err(|_err| Error::InvalidData)
    }

    fn copy_into(&self, buf: &mut [u8]) {
        buf[0..self.metadata.frame_size()].copy_from_slice(self.data);
    }

    fn to_owned(&self) -> FramedOwnedMessage {
        FramedOwnedMessage::new(self.data.to_vec())
            .expect("Couldn't create owned message from slice, which shouldn't be possible")
    }
}

/// A framed typed message coming from a slice of bytes that wraps a `FramedSliceMessage` with annotated type.
pub struct FramedSliceTypedMessage<'a, T>
where
    T: MessageType<'a>,
{
    message: FramedSliceMessage<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> FramedSliceTypedMessage<'a, T>
where
    T: MessageType<'a>,
{
    pub fn new(data: &'a [u8]) -> Result<FramedSliceTypedMessage<'a, T>, Error> {
        let expected_type = <T as MessageType>::message_type();
        let message = FramedSliceMessage::new(data)?;
        if message.message_type() != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type,
                message.message_type()
            );
            return Err(Error::InvalidData);
        }

        Ok(FramedSliceTypedMessage {
            message,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_from_next_offset(
        data: &'a [u8],
        next_offset: usize,
    ) -> Result<FramedSliceTypedMessage<'a, T>, Error> {
        let expected_type = <T as MessageType>::message_type();
        let message = FramedSliceMessage::new_from_next_offset(data, next_offset)?;
        if message.message_type() != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type,
                message.message_type()
            );
            return Err(Error::InvalidData);
        }

        Ok(FramedSliceTypedMessage {
            message,
            phantom: std::marker::PhantomData,
        })
    }
}

impl<'a, T> FramedTypedMessage<T> for FramedSliceTypedMessage<'a, T>
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

    fn to_owned(&self) -> FramedOwnedTypedMessage<T> {
        let owned_message = self.message.to_owned();
        owned_message.into_typed()
    }
}

/// Iterator on a stream of untyped framed messages.
/// Will return None on error, and the `last_error` field will identify the error, if any.
pub struct FramedMessageIterator<'a> {
    buffer: &'a [u8],
    current_offset: usize,
    pub last_error: Option<Error>,
}

impl<'a> FramedMessageIterator<'a> {
    pub fn new(buffer: &'a [u8]) -> FramedMessageIterator<'a> {
        FramedMessageIterator {
            buffer,
            current_offset: 0,
            last_error: None,
        }
    }
}

impl<'a> Iterator for FramedMessageIterator<'a> {
    type Item = IteratedFramedSliceMessage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset;
        let slice = &self.buffer[offset..];
        match FramedSliceMessage::new(slice) {
            Ok(framed_message) => {
                self.current_offset += framed_message.frame_size();
                Some(IteratedFramedSliceMessage {
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

pub struct IteratedFramedSliceMessage<'a> {
    pub offset: usize,
    pub framed_message: FramedSliceMessage<'a>,
}

/// A standalone framed message.
///
/// Uses a OwningHandle in order to prevent having data twice in memory and use a FramedSliceMessage
/// that references data that is stored in struct itself.
///
/// See https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct
/// As noted here: https://github.com/Kimundi/owning-ref-rs/issues/27
/// We should never expose the 'static lifetime through the API because it may lead into unsafe
/// behaviour.
pub struct FramedOwnedMessage {
    owned_slice_message: OwningHandle<Vec<u8>, Box<FramedSliceMessage<'static>>>,
}

impl FramedOwnedMessage {
    pub fn new(data: Vec<u8>) -> Result<FramedOwnedMessage, Error> {
        let owned_slice_message = OwningHandle::try_new(data, |data| unsafe {
            FramedSliceMessage::new(data.as_ref().unwrap()).map(Box::new)
        })?;

        Ok(FramedOwnedMessage {
            owned_slice_message,
        })
    }

    pub fn from_builder<A: Allocator>(
        message_type: u16,
        builder: &Builder<A>,
    ) -> Result<FramedOwnedMessage, Error> {
        let mut writer = OwnedFramedMessageWriter::new(message_type);
        capnp::serialize::write_message(&mut writer, builder).unwrap();
        let (buffer, _metadata) = writer.finish()?;

        FramedOwnedMessage::new(buffer)
    }

    pub fn into_typed<T>(self) -> FramedOwnedTypedMessage<T>
    where
        T: for<'a> MessageType<'a>,
    {
        FramedOwnedTypedMessage {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl Clone for FramedOwnedMessage {
    fn clone(&self) -> Self {
        FramedOwnedMessage::new(self.owned_slice_message.data.to_vec()).unwrap()
    }
}

impl FramedMessage for FramedOwnedMessage {
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

    fn to_owned(&self) -> FramedOwnedMessage {
        FramedOwnedMessage::new(self.owned_slice_message.data.to_vec()).unwrap()
    }
}

/// A standalone framed typed message that wraps a `FramedOwnedMessage` with annotated type.
#[derive(Clone)]
pub struct FramedOwnedTypedMessage<T>
where
    T: for<'a> MessageType<'a>,
{
    message: FramedOwnedMessage,
    phantom: std::marker::PhantomData<T>,
}

impl<T> FramedTypedMessage<T> for FramedOwnedTypedMessage<T>
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

    fn to_owned(&self) -> FramedOwnedTypedMessage<T> {
        let owned_message = self.message.clone();
        owned_message.into_typed()
    }
}

/// Framed message writer that wraps a slice, that should have enough capacity, and exposes a Write implementation used by capnp
struct FramedMessageWriter<'a> {
    message_type: u16,
    buffer: &'a mut [u8],
    count: usize,
    finished: bool,
}

impl<'a> FramedMessageWriter<'a> {
    fn new(message_type: u16, buffer: &'a mut [u8]) -> FramedMessageWriter<'a> {
        FramedMessageWriter {
            message_type,
            buffer,
            count: FrameMetadata::SIZE,
            finished: false,
        }
    }

    fn finish(&mut self) -> Result<FrameMetadata, Error> {
        let message_size = self.count - FrameMetadata::SIZE;

        let metadata = FrameMetadata {
            message_type: self.message_type,
            message_size,
            signature_size: 0,
        };

        metadata.copy_into_slice(&mut self.buffer[0..])?;
        metadata.copy_into_slice(&mut self.buffer[metadata.footer_offset()..])?;
        self.finished = true;

        Ok(metadata)
    }
}

impl<'a> Drop for FramedMessageWriter<'a> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish();
        }
    }
}

impl<'a> std::io::Write for FramedMessageWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let offset_from = self.count;
        let len = buf.len();
        let offset_to = offset_from + len;

        if offset_to > self.buffer.len() {
            error!(
                "Tried to write a message that exceeded size of buffer: offset_to={} buffer_len={}",
                offset_to,
                self.buffer.len()
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Message bigger than buffer len",
            ));
        }

        self.buffer[offset_from..offset_to].copy_from_slice(buf);
        self.count += len;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

/// Helper method that writes a single message into a buffer. Uses a `FramedMessageWriter`
pub fn write_framed_builder_into_buffer<A: capnp::message::Allocator>(
    buffer: &mut [u8],
    message_type: u16,
    message_builder: &capnp::message::Builder<A>,
) -> Result<FrameMetadata, Error> {
    let mut framed_writer = FramedMessageWriter::new(message_type, buffer);
    capnp::serialize::write_message(&mut framed_writer, &message_builder)?;
    framed_writer.finish()
}

/// Framed message writer that writes into an owned Vector (and resizes itself), and exposes a Write implementation used by capnp
pub struct OwnedFramedMessageWriter {
    message_type: u16,
    buffer: Vec<u8>,
    count: usize,
}

impl OwnedFramedMessageWriter {
    pub fn new(message_type: u16) -> OwnedFramedMessageWriter {
        let mut buffer = Vec::new();
        Self::push_empty_bytes(&mut buffer, FrameMetadata::SIZE);
        OwnedFramedMessageWriter {
            message_type,
            buffer,
            count: FrameMetadata::SIZE,
        }
    }

    pub fn finish(mut self) -> Result<(Vec<u8>, FrameMetadata), Error> {
        let message_size = self.count - FrameMetadata::SIZE;

        let metadata = FrameMetadata {
            message_type: self.message_type,
            message_size,
            signature_size: 0,
        };

        metadata.copy_into_slice(&mut self.buffer[0..])?;

        // push empty bytes for footer meta & write it
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

impl<'a> std::io::Write for OwnedFramedMessageWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        for elem in buf {
            self.buffer.push(*elem);
        }
        self.count += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[derive(Fail, Debug, Clone, PartialEq)]
#[fail(display = "A message serialization error occurred")]
pub enum Error {
    #[fail(display = "Couldn't deserialization data")]
    InvalidData,
    #[fail(display = "Invalid message size")]
    InvalidSize,
    #[fail(display = "Destination size is too small")]
    DestinationSize,
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

        let message_type = unpack_u16(&buffer[0..Self::TYPE_FIELD_SIZE]);
        let message_size = unpack_u32(&buffer[2..2 + Self::DATA_FIELD_SIZE]) as usize;
        let signature_size = usize::from(unpack_u16(&buffer[6..6 + Self::SIG_FIELD_SIZE]));

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

        pack_u16(self.message_type, &mut buffer[0..Self::TYPE_FIELD_SIZE]);
        pack_u32(
            self.message_size as u32,
            &mut buffer[2..2 + Self::DATA_FIELD_SIZE],
        );
        pack_u16(
            self.signature_size as u16,
            &mut buffer[6..6 + Self::SIG_FIELD_SIZE],
        );

        Ok(())
    }

    #[inline]
    fn frame_size(&self) -> usize {
        Self::SIZE + Self::SIZE + usize::from(self.message_size) + usize::from(self.signature_size)
    }

    #[inline]
    fn message_offset(&self) -> usize {
        Self::SIZE
    }

    #[inline]
    fn footer_offset(&self) -> usize {
        Self::SIZE + usize::from(self.message_size) + usize::from(self.signature_size)
    }

    #[inline]
    fn message_range(&self) -> std::ops::Range<usize> {
        let message_offset = self.message_offset();
        message_offset..message_offset + self.message_size
    }

    #[allow(dead_code)]
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

#[inline]
fn pack_u32(value: u32, into: &mut [u8]) {
    debug_assert!(into.len() >= 4);
    byteorder::LittleEndian::write_u32(into, value);
}

#[inline]
fn unpack_u32(from: &[u8]) -> u32 {
    debug_assert!(from.len() >= 4);
    byteorder::LittleEndian::read_u32(from)
}

#[inline]
fn pack_u16(value: u16, into: &mut [u8]) {
    debug_assert!(into.len() >= 2);
    byteorder::LittleEndian::write_u16(into, value);
}

#[inline]
fn unpack_u16(from: &[u8]) -> u16 {
    debug_assert!(from.len() >= 2);
    byteorder::LittleEndian::read_u16(from)
}

#[cfg(test)]
pub mod tests {
    use crate::data_chain_capnp::{block, entry_header};

    use super::*;

    #[test]
    fn pack_unpack_u32() {
        let mut buf = vec![0, 0, 0, 0, 0, 0];
        pack_u32(44323213, &mut buf);
        assert_eq!(44323213, unpack_u32(&buf));
    }

    #[test]
    fn pack_unpack_u16() {
        let mut buf = vec![0, 0, 0];
        pack_u16(1613, &mut buf);
        assert_eq!(1613, unpack_u16(&buf));
    }

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
    fn message_builder_into_data() {
        let test_block_builder = build_test_block("block_hash");

        let message_data = test_block_builder.into_framed_vec().unwrap();
        let slice_message = FramedSliceMessage::new(&message_data).unwrap();
        assert_eq!(
            slice_message.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        let typed_message = slice_message.into_typed::<block::Owned>();
        let reader = typed_message.get_typed_reader().unwrap();
        assert_eq!(reader.get_hash().unwrap(), "block_hash");
        assert_eq!(typed_message.frame_size(), message_data.len());
    }

    #[test]
    fn message_builder_into_owned() {
        let test_block_builder = build_test_block("block_hash");

        let block_owned_message = test_block_builder.as_owned_framed().unwrap();
        assert_eq!(
            block_owned_message.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        let message_reader = block_owned_message.get_typed_reader().unwrap();
        assert_eq!(message_reader.get_hash().unwrap(), "block_hash");
    }

    #[test]
    fn message_builder_write_into_buffer() {
        let test_block_builder = build_test_block("block_hash");

        let mut data = [0u8; 1000];
        let metadata = test_block_builder.write_into(&mut data).unwrap();

        let framed_data = test_block_builder.into_framed_vec().unwrap();
        assert_eq!(&framed_data[..], &data[..metadata.frame_size()]);
    }

    #[test]
    fn framed_slice_message_invalid_message() {
        // no data found
        let data = [0u8; 1000];
        assert_eq!(FramedSliceMessage::new(&data).err(), Some(Error::EOF));

        // invalid size
        let mut data = [0u8; 1000];
        pack_u32(10, &mut data);
        assert!(FramedSliceMessage::new(&data).is_err());

        // overflow size
        let mut data = [0u8; 1000];
        pack_u32(10000, &mut data);
        assert!(FramedSliceMessage::new(&data).is_err());
    }

    #[test]
    fn framed_slice_message_from_next_offset() -> Result<(), Error> {
        let mut data = [0u8; 1000];

        let mut block1_builder = build_test_block("block1_hash");
        let block1_metadata =
            write_framed_builder_into_buffer(&mut data[0..], 123, &block1_builder.get_builder())?;

        let block2_offset = block1_metadata.frame_size();
        let mut block2_builder = build_test_block("block2_hash");
        let block2_metadata = write_framed_builder_into_buffer(
            &mut data[block2_offset..],
            456,
            &block2_builder.get_builder(),
        )?;

        let block3_offset = block2_offset + block2_metadata.frame_size();
        let mut block3_builder = build_test_block("block3_hash");
        let block3_metadata = write_framed_builder_into_buffer(
            &mut data[block3_offset..],
            789,
            &block3_builder.get_builder(),
        )?;

        // wrong offset tests
        assert!(FramedSliceMessage::new_from_next_offset(&data[0..], 0).is_err());
        assert!(FramedSliceMessage::new_from_next_offset(&data[0..], 100).is_err());

        let frame =
            FramedSliceMessage::new_from_next_offset(&data[0..], block1_metadata.frame_size())?;
        assert_eq!(frame.message_type(), 123);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_hash()?, "block1_hash");

        let frame = FramedSliceMessage::new_from_next_offset(&data[0..], block3_offset)?;
        assert_eq!(frame.message_type(), 456);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_hash()?, "block2_hash");

        let frame = FramedSliceMessage::new_from_next_offset(
            &data[0..],
            block3_offset + block3_metadata.frame_size(),
        )?;
        assert_eq!(frame.message_type(), 789);
        let block_reader = frame.get_typed_reader::<block::Owned>()?;
        assert_eq!(block_reader.get_hash()?, "block3_hash");

        Ok(())
    }

    #[test]
    fn write_framed_builder_fail_not_enough_space() {
        let mut block_builder = build_test_block("block_hash");

        let mut data = [0u8; 10];
        assert!(
            write_framed_builder_into_buffer(&mut data, 123, block_builder.get_builder()).is_err()
        );
    }

    #[test]
    fn framed_message_iterator() {
        let mut data = [0u8; 500_000];

        let mut next_offset = 0;
        for i in 0..1000 {
            let mut block_builder = build_test_block(&format!("block{}", i));

            let metadata = write_framed_builder_into_buffer(
                &mut data[next_offset..],
                123,
                block_builder.get_builder(),
            )
            .unwrap();
            next_offset += metadata.frame_size();
        }

        // simple forward iteration
        let mut iterator = FramedMessageIterator::new(&data);
        let mut last_offset = 0;
        for message in iterator.by_ref() {
            assert!(last_offset == 0 || message.offset > last_offset);
            last_offset = message.offset;
        }
        assert_eq!(iterator.last_error, None);

        // make sure we can deserialize
        let message = FramedMessageIterator::new(&data).take(1).last().unwrap();
        assert_eq!(message.offset, 0);
        let typed_reader = message.framed_message.into_typed::<block::Owned>();
        let block_reader = typed_reader.get_typed_reader().unwrap();
        assert_eq!(block_reader.get_hash().unwrap(), "block0");

        // iterator typing
        let message = FramedMessageIterator::new(&data).take(10);
        let hashes: Vec<String> = message
            .filter(|m| m.framed_message.message_type() == 123)
            .map(|m| m.framed_message.into_typed::<block::Owned>())
            .map(|b| {
                b.get_typed_reader()
                    .unwrap()
                    .get_hash()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(
            hashes,
            vec![
                "block0", "block1", "block2", "block3", "block4", "block5", "block6", "block7",
                "block8", "block9",
            ]
        );
    }

    #[test]
    fn framed_message_iterator_error() {
        // invalid data should have an error
        let mut data = [0u8; 1000];
        pack_u16(10, &mut data);
        pack_u32(42, &mut data[2..]);
        let mut iterator = FramedMessageIterator::new(&data);
        assert_eq!(iterator.by_ref().count(), 0);
        assert!(iterator.last_error.is_some());
    }

    fn build_test_block(hash: &str) -> MessageBuilder<block::Owned> {
        let mut block_msg_builder = MessageBuilder::<block::Owned>::new();

        let mut block_builder = block_msg_builder.get_builder_typed();
        block_builder.set_hash(hash);

        let mut entries = block_builder.init_entries(1);
        {
            let mut entry = entries.get(0);

            let mut entry_header_msg_builder = MessageBuilder::<entry_header::Owned>::new();
            let mut header_builder = entry_header_msg_builder.get_builder_typed();
            header_builder.set_hash(hash);

            entry.set_header(header_builder.into_reader()).unwrap();
        }

        block_msg_builder
    }
}
