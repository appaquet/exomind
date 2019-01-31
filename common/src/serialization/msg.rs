use std;

use byteorder;
use byteorder::ByteOrder;
use capnp;
pub use capnp::message::ReaderSegments;
use capnp::message::{Allocator, Builder, HeapAllocator, Reader};
use capnp::serialize::SliceSegments;
use lazycell::LazyCell;
use owning_ref::OwningHandle;

// TODO: Use ScratchSpace to reuse memory buffer for writes
// TODO: usize is not compatible with 32bit

const FRAMING_DATA_SIZE: usize = 4;
const FRAMING_TYPE_SIZE: usize = 2;
const FRAMING_HEADER_SIZE: usize = FRAMING_TYPE_SIZE + FRAMING_DATA_SIZE;
const FRAMING_FOOTER_SIZE: usize = FRAMING_DATA_SIZE;

/// Trait that needs to have an impl for each capnp generated message struct.
/// Used to identify a unique type id for each message and annotate each framed message.
pub trait MessageType<'a>: capnp::traits::Owned<'a> {
    fn message_type() -> u16;
}

/// A Framed Message is a capnp message with an extra header identifying the message type and message size.
pub trait FramedMessage {
    fn message_type(&self) -> u16;
    fn message_size(&self) -> usize;
    fn data_size(&self) -> usize;
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
    fn data_size(&self) -> usize;
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
        let (buffer, _message_size, _data_size) = writer.finish();
        Ok(buffer)
    }

    pub fn write_into(&self, data: &mut [u8]) -> Result<usize, Error> {
        let (_message_size, data_size) =
            write_framed_builder_into_buffer(data, self.message_type, &self.builder).map_err(
                |err| {
                    error!("Couldn't write builder into buffer: {:?}", err);
                    Error::InvalidSize
                },
            )?;
        Ok(data_size)
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
    message_type: u16,
    message_size: usize,
    data_size: usize,
    data: &'a [u8],
    lazy_reader: LazyCell<Reader<SliceSegments<'a>>>,
}

impl<'a> FramedSliceMessage<'a> {
    pub fn new(data: &[u8]) -> Result<FramedSliceMessage, Error> {
        let (message_type, message_size, data_size) = read_framed_message_meta(data)?;

        Ok(FramedSliceMessage {
            message_type,
            message_size,
            data_size,
            data,
            lazy_reader: LazyCell::new(),
        })
    }

    pub fn new_from_next_offset(
        data: &[u8],
        next_offset: usize,
    ) -> Result<FramedSliceMessage, Error> {
        if data.len() < next_offset || next_offset < FRAMING_FOOTER_SIZE {
            error!(
                "Tried to read from next offset {} in buffer of len {}",
                next_offset,
                data.len()
            );
            return Err(Error::InvalidSize);
        }

        let footer_offset = next_offset - FRAMING_DATA_SIZE;
        let footer_message_size =
            unpack_u32(&data[footer_offset..footer_offset + FRAMING_DATA_SIZE]) as usize;
        let footer_total_size = footer_message_size + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE;
        if footer_total_size > next_offset {
            error!("End frame size would exceed buffer 0th position (footer_total_size={} > next_offset={})", next_offset, footer_total_size);
            return Err(Error::InvalidSize);
        }

        Self::new(&data[(next_offset - footer_total_size)..next_offset])
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

    pub fn data_size(&self) -> usize {
        self.data_size
    }
}

impl<'a> FramedMessage for FramedSliceMessage<'a> {
    fn message_type(&self) -> u16 {
        self.message_type
    }

    fn message_size(&self) -> usize {
        self.message_size
    }

    fn data_size(&self) -> usize {
        self.data_size
    }

    fn get_typed_reader<'b, T: MessageType<'b>>(
        &'b self,
    ) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        let reader = self.lazy_reader.try_borrow_with(|| {
            let offset_from = FRAMING_HEADER_SIZE;
            let offset_to = offset_from + self.message_size;
            Self::read_capn_message(&self.data[offset_from..offset_to])
        })?;

        reader.get_root().map_err(|_err| Error::InvalidData)
    }

    fn copy_into(&self, buf: &mut [u8]) {
        buf[0..self.data_size].copy_from_slice(self.data);
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
        if message.message_type != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type, message.message_type
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
        if message.message_type != expected_type {
            error!(
                "Trying to read a message of type {}, but got type {} in buffer",
                expected_type, message.message_type
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

    fn data_size(&self) -> usize {
        self.message.data_size()
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
                self.current_offset += framed_message.data_size;
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
        let (buffer, _message_size, _data_size) = writer.finish();

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
        self.owned_slice_message.message_type
    }

    fn message_size(&self) -> usize {
        self.owned_slice_message.message_size
    }

    fn data_size(&self) -> usize {
        self.owned_slice_message.data_size
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

    fn data_size(&self) -> usize {
        self.message.data_size()
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
            count: FRAMING_HEADER_SIZE,
            finished: false,
        }
    }

    fn finish(&mut self) -> (usize, usize) {
        let message_size = self.count - FRAMING_HEADER_SIZE;

        // write header
        pack_u16(self.message_type, self.buffer);
        pack_u32(message_size as u32, &mut self.buffer[FRAMING_TYPE_SIZE..]);

        // write footer
        pack_u32(
            message_size as u32,
            &mut self.buffer[self.count..(self.count + FRAMING_DATA_SIZE)],
        );

        self.finished = true;
        (
            message_size,
            message_size + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE,
        )
    }
}

impl<'a> Drop for FramedMessageWriter<'a> {
    fn drop(&mut self) {
        if !self.finished {
            self.finish();
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
) -> std::io::Result<(usize, usize)> {
    let mut framed_writer = FramedMessageWriter::new(message_type, buffer);
    capnp::serialize::write_message(&mut framed_writer, &message_builder)?;
    let (message_size, data_size) = framed_writer.finish();
    Ok((message_size, data_size))
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
        Self::push_empty_bytes(&mut buffer, FRAMING_HEADER_SIZE);
        OwnedFramedMessageWriter {
            message_type,
            buffer,
            count: FRAMING_HEADER_SIZE,
        }
    }

    pub fn finish(mut self) -> (Vec<u8>, usize, usize) {
        let message_size = self.count - FRAMING_HEADER_SIZE;

        // write header
        pack_u16(self.message_type, &mut self.buffer[0..FRAMING_TYPE_SIZE]);
        pack_u32(message_size as u32, &mut self.buffer[FRAMING_TYPE_SIZE..]);

        // write footer
        Self::push_empty_bytes(&mut self.buffer, FRAMING_FOOTER_SIZE);
        pack_u32(
            message_size as u32,
            &mut self.buffer[self.count..(self.count + FRAMING_DATA_SIZE)],
        );

        let data_size = message_size + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE;
        let _buf_size = self.buffer.len();
        (self.buffer, message_size, data_size)
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

#[derive(Fail, Debug, Clone, Copy, PartialEq)]
#[fail(display = "A message serialization error occurred")]
pub enum Error {
    #[fail(display = "Couldn't deserialization data")]
    InvalidData,
    #[fail(display = "Invalid message size")]
    InvalidSize,
    #[fail(display = "Reached end of message / stream")]
    EOF,
}

fn read_framed_message_meta(buffer: &[u8]) -> Result<(u16, usize, usize), Error> {
    if buffer.len() < FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE {
        return Err(Error::EOF);
    }

    let msg_type = unpack_u16(&buffer[0..FRAMING_TYPE_SIZE]);
    let msg_size_begin =
        unpack_u32(&buffer[FRAMING_TYPE_SIZE..(FRAMING_TYPE_SIZE + FRAMING_DATA_SIZE)]) as usize;
    let data_size = msg_size_begin + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE;
    if buffer.len() >= data_size {
        let footer_offset = FRAMING_HEADER_SIZE + msg_size_begin;
        let msg_size_end =
            unpack_u32(&buffer[footer_offset..(footer_offset + FRAMING_DATA_SIZE)]) as usize;
        if msg_size_begin != 0 && msg_size_begin == msg_size_end {
            Ok((msg_type, msg_size_begin, data_size))
        } else if msg_size_begin == 0 {
            Err(Error::EOF)
        } else {
            warn!(
                "Invalid frame size information: begin {} != end {}",
                msg_size_begin, msg_size_end
            );
            Err(Error::InvalidSize)
        }
    } else {
        warn!(
            "Invalid frame size: would exceed buffer size: msg_size_begin={} data_size={} buffer_size={}",
            msg_size_begin,
            data_size,
            buffer.len()
        );
        Err(Error::InvalidSize)
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
    use super::*;
    use crate::data_chain_capnp::{block, entry_header};

    #[test]
    fn test_pack_unpack_u32() {
        let mut buf = vec![0, 0, 0, 0, 0, 0];
        pack_u32(44323213, &mut buf);
        assert_eq!(44323213, unpack_u32(&buf));
    }

    #[test]
    fn test_pack_unpack_u16() {
        let mut buf = vec![0, 0, 0];
        pack_u16(1613, &mut buf);
        assert_eq!(1613, unpack_u16(&buf));
    }

    #[test]
    fn test_message_builder_into_data() {
        let test_block_builder = build_test_block();

        let message_data = test_block_builder.into_framed_vec().unwrap();
        let slice_message = FramedSliceMessage::new(&message_data).unwrap();
        assert_eq!(
            slice_message.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        let typed_message = slice_message.into_typed::<block::Owned>();
        let reader = typed_message.get_typed_reader().unwrap();
        assert_eq!(reader.get_hash().unwrap(), "block_hash");

        assert_eq!(typed_message.data_size(), message_data.len());
    }

    #[test]
    fn test_message_builder_into_owned() {
        let test_block_builder = build_test_block();

        let block_owned_message = test_block_builder.as_owned_framed().unwrap();
        assert_eq!(
            block_owned_message.message_type(),
            <block::Owned as MessageType>::message_type()
        );

        {
            let message_reader = block_owned_message.get_typed_reader().unwrap();
            assert_eq!(message_reader.get_hash().unwrap(), "block_hash");
        }
    }

    #[test]
    fn test_message_builder_write_into_buffer() {
        let test_block_builder = build_test_block();

        let mut data = [0u8; 1000];
        let data_size = test_block_builder.write_into(&mut data).unwrap();

        let framed_data = test_block_builder.into_framed_vec().unwrap();
        assert_eq!(&framed_data[..], &data[..data_size]);
    }

    #[test]
    fn test_framed_slice_message_invalid_message() {
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
    fn test_framed_slice_message_from_next_offset() {
        let mut test_block_builder = build_test_block();
        let message_builder = test_block_builder.get_builder();

        let mut data = [0u8; 1000];
        let (_msg_size, data_size) =
            write_framed_builder_into_buffer(&mut data, 123, &message_builder).unwrap();

        let (_msg_size, data_size) =
            write_framed_builder_into_buffer(&mut data[data_size..], 456, &message_builder)
                .unwrap();

        let (_write_msg_size, write_data_size) =
            write_framed_builder_into_buffer(&mut data[2 * data_size..], 789, &message_builder)
                .unwrap();

        assert!(FramedSliceMessage::new_from_next_offset(&data[0..], 0).is_err());
        assert!(FramedSliceMessage::new_from_next_offset(&data[0..], 100).is_err());

        let read_msg =
            FramedSliceMessage::new_from_next_offset(&data[0..], write_data_size).unwrap();
        assert_eq!(read_msg.message_type(), 123);

        let read_msg =
            FramedSliceMessage::new_from_next_offset(&data[0..], 2 * write_data_size).unwrap();
        assert_eq!(read_msg.message_type(), 456);
    }

    #[test]
    fn test_write_framed_builder_fail_not_enough_space() {
        let mut block_builder = build_test_block();

        let mut data = [0u8; 10];
        assert!(
            write_framed_builder_into_buffer(&mut data, 123, block_builder.get_builder()).is_err()
        );
    }

    #[test]
    fn test_framed_message_iterator() {
        let mut data = [0u8; 500_000];

        let mut next_offset = 0;
        for _i in 0..1000 {
            let mut block_builder = build_test_block();

            let (_msg_size, data_size) = write_framed_builder_into_buffer(
                &mut data[next_offset..],
                123,
                block_builder.get_builder(),
            )
            .unwrap();
            next_offset += data_size;
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

        assert_eq!(block_reader.get_hash().unwrap(), "block_hash");

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
        assert_eq!(hashes.len(), 10);

        // invalid data should have an error
        let mut data = [0u8; 1000];
        pack_u16(10, &mut data);
        pack_u32(42, &mut data[FRAMING_TYPE_SIZE..]);
        let mut iterator = FramedMessageIterator::new(&data);
        assert_eq!(iterator.by_ref().count(), 0);
        assert!(iterator.last_error.is_some());
    }

    fn build_test_block() -> MessageBuilder<block::Owned> {
        let mut block_msg_builder = MessageBuilder::<block::Owned>::new();

        let mut block_builder = block_msg_builder.get_builder_typed();
        block_builder.set_hash("block_hash");

        let mut entries = block_builder.init_entries(1);
        {
            let mut entry = entries.get(0);

            let mut entry_header_msg_builder = MessageBuilder::<entry_header::Owned>::new();
            let mut header_builder = entry_header_msg_builder.get_builder_typed();
            header_builder.set_hash("entry_hash");

            entry.set_header(header_builder.into_reader()).unwrap();
        }

        block_msg_builder
    }
}
