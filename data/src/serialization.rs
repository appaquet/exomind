use super::*;

use std;

use byteorder;
use byteorder::ByteOrder;

use capnp;
use capnp::message::{Allocator, Builder, HeapAllocator, Reader, ReaderOptions};
use capnp::serialize::{OwnedSegments, SliceSegments};
use capnp::traits::FromPointerReader;

pub use capnp::message::ReaderSegments;

// TODO: Magic bytes
// TODO: Use ScratchSpace to reuse memory buffer for writes
// TODO: usize is not compatible with 32bit

const FRAMING_DATA_SIZE: usize = 4;
const FRAMING_TYPE_SIZE: usize = 2;
const FRAMING_HEADER_SIZE: usize = FRAMING_TYPE_SIZE + FRAMING_DATA_SIZE;
const FRAMING_FOOTER_SIZE: usize = FRAMING_DATA_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidData,
    EOF,
}

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
        match serialization::FramedSliceMessage::new(slice) {
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
            },
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

pub trait FramedMessage<S: ReaderSegments> {
    fn message_size(&self) -> usize;
    fn data_size(&self) -> usize;
    fn data(&self) -> &[u8];
    fn get_reader(&self) -> &Reader<S>;
    fn get_root<'b, T: FromPointerReader<'b>>(&'b self) -> Result<T, Error>;

    fn copy_into(&self, buf: &mut [u8]) {
        let data = self.data();
        buf[0..data.len()].copy_from_slice(data);
    }
}

pub trait FramedTypedMessage<T>
where
    T: for<'a> capnp::traits::Owned<'a>,
{
    fn message_size(&self) -> usize;
    fn data_size(&self) -> usize;
    fn data(&self) -> &[u8];
    fn get(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error>;

    fn copy_into(&self, buf: &mut [u8]) {
        let data = self.data();
        buf[0..data.len()].copy_from_slice(data);
    }
}

pub struct FramedSliceMessage<'a> {
    message_size: usize,
    data_size: usize,
    data: &'a [u8],
    reader: Reader<SliceSegments<'a>>,
}

impl<'a> FramedSliceMessage<'a> {
    pub fn new(data: &[u8]) -> Result<FramedSliceMessage, Error> {
        let (message_size, data_size) = get_framed_message_size(data)?;

        let offset_from = FRAMING_HEADER_SIZE;
        let offset_to = offset_from + message_size;
        let reader = Self::read_capn_message(&data[offset_from..offset_to])?;

        Ok(FramedSliceMessage {
            message_size,
            data_size,
            data,
            reader,
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
        FramedOwnedMessage::new(self.message_size, self.data_size, &self.data).unwrap()
    }

    pub fn into_typed<T>(self) -> FramedSliceTypedMessage<'a, T>
    where
        T: capnp::traits::Owned<'a>,
    {
        FramedSliceTypedMessage {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> FramedMessage<SliceSegments<'a>> for FramedSliceMessage<'a> {
    fn message_size(&self) -> usize {
        self.message_size
    }

    fn data_size(&self) -> usize {
        self.data_size
    }

    fn data(&self) -> &[u8] {
        self.data
    }

    fn get_reader(&self) -> &Reader<SliceSegments<'a>> {
        &self.reader
    }

    fn get_root<'b, T: FromPointerReader<'b>>(&'b self) -> Result<T, Error> {
        let reader = self.get_reader();
        reader.get_root().map_err(|err| Error::InvalidData)
    }
}

pub struct FramedSliceTypedMessage<'a, T>
where
    T: capnp::traits::Owned<'a>,
{
    message: FramedSliceMessage<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> FramedTypedMessage<T> for FramedSliceTypedMessage<'a, T>
where
    T: for<'b> capnp::traits::Owned<'b>,
{
    fn message_size(&self) -> usize {
        self.message.message_size()
    }

    fn data_size(&self) -> usize {
        self.message.data_size()
    }

    fn data(&self) -> &[u8] {
        self.message.data
    }

    fn get(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        let reader = self.message.get_reader();
        reader.get_root().map_err(|err| {
            error!("Couldn't get root from owned reader");
            Error::InvalidData
        })
    }
}

pub struct FramedOwnedMessage {
    message_size: usize,
    data_size: usize,
    data: Vec<u8>,
    reader: Reader<OwnedSegments>,
}

impl FramedOwnedMessage {
    pub fn new(
        message_size: usize,
        data_size: usize,
        data: &[u8],
    ) -> Result<FramedOwnedMessage, Error> {
        // TODO: This is ineficient... We have twice the data in memory
        let data = data.to_vec();
        let opts = ReaderOptions::new();
        let reader = {
            let from_offset = FRAMING_HEADER_SIZE;
            let to_offset = data.len() - FRAMING_FOOTER_SIZE;
            let mut cursor = std::io::Cursor::new(&data[from_offset..to_offset]);
            capnp::serialize::read_message(&mut cursor, opts).unwrap()
        };

        Ok(FramedOwnedMessage {
            message_size,
            data_size,
            data,
            reader,
        })
    }

    pub fn from_builder<A: Allocator>(builder: &Builder<A>) -> Result<FramedOwnedMessage, Error> {
        // TODO: This is not efficient, we could just write data to buffer & use builder as reader instead of re-reading

        let mut writer = OwnedFramedMessageWriter::new();
        capnp::serialize::write_message(&mut writer, &builder).map_err(|err| {
            error!(
                "Error writing message to OwnedFramedMessageWriter: {:?}",
                err
            );
            Error::InvalidData
        })?;
        let (buffer, message_size, data_size) = writer.finish()?;

        // TODO: not efficient... it re-reads
        FramedOwnedMessage::new(message_size, data_size, &buffer)
    }

    pub fn into_typed<T>(self) -> FramedOwnedTypedMessage<T>
    where
        T: for<'a> capnp::traits::Owned<'a>,
    {
        FramedOwnedTypedMessage {
            message: self,
            phantom: std::marker::PhantomData,
        }
    }
}

impl FramedMessage<OwnedSegments> for FramedOwnedMessage {
    fn message_size(&self) -> usize {
        self.message_size
    }

    fn data_size(&self) -> usize {
        self.data_size
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn get_reader(&self) -> &Reader<OwnedSegments> {
        &self.reader
    }

    fn get_root<'b, T: FromPointerReader<'b>>(&'b self) -> Result<T, Error> {
        let reader = self.get_reader();
        reader.get_root().map_err(|err| Error::InvalidData)
    }
}

pub struct FramedOwnedTypedMessage<T>
where
    T: for<'a> capnp::traits::Owned<'a>,
{
    message: FramedOwnedMessage,
    phantom: std::marker::PhantomData<T>,
}

impl<T> FramedTypedMessage<T> for FramedOwnedTypedMessage<T>
where
    T: for<'a> capnp::traits::Owned<'a>,
{
    fn message_size(&self) -> usize {
        self.message.message_size()
    }

    fn data_size(&self) -> usize {
        self.message.data_size()
    }

    fn data(&self) -> &[u8] {
        &self.message.data
    }

    fn get(&self) -> Result<<T as capnp::traits::Owned>::Reader, Error> {
        let reader = self.message.get_reader();
        reader.get_root().map_err(|err| {
            error!("Couldn't get root from owned reader");
            Error::InvalidData
        })
    }
}

pub struct FramedMessageBuilder<T>
where
    T: for<'a> capnp::traits::Owned<'a>,
{
    builder: Builder<HeapAllocator>, //  TODO: We should use reusable scratch space allocator
    phantom: std::marker::PhantomData<T>,
}

impl<T> FramedMessageBuilder<T>
where
    T: for<'a> capnp::traits::Owned<'a>,
{
    pub fn new() -> FramedMessageBuilder<T> {
        let builder = Builder::new_default();
        FramedMessageBuilder {
            builder,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn init_root(&mut self) -> <T as capnp::traits::Owned>::Builder {
        self.builder.init_root()
    }

    pub fn get_root_builder(&mut self) -> Result<<T as capnp::traits::Owned>::Builder, Error> {
        self.builder.get_root().map_err(|err| {
            error!("Couldn't get root builder from framed message builder. init_root() wasn't called first?");
            Error::InvalidData
        })
    }

    pub fn get_root_reader(&mut self) -> Result<<T as capnp::traits::Owned>::Builder, Error> {
        self.builder.get_root().map_err(|err| {
            error!("Couldn't get root builder from framed message builder. init_root() wasn't called first?");
            Error::InvalidData
        })
    }

    pub fn into_framed(self) -> Result<FramedOwnedTypedMessage<T>, Error> {
        let msg = FramedOwnedMessage::from_builder(&self.builder)?;
        Ok(msg.into_typed())
    }
}

pub struct FramedMessageWriter<'a> {
    buffer: &'a mut [u8],
    count: usize,
    finished: bool,
}

impl<'a> FramedMessageWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> FramedMessageWriter<'a> {
        FramedMessageWriter {
            buffer,
            count: FRAMING_HEADER_SIZE,
            finished: false,
        }
    }

    pub fn finish(&mut self) -> (usize, usize) {
        let message_size = self.count - FRAMING_HEADER_SIZE;

        // write header
        pack_u16(0, self.buffer);
        pack_u32(message_size as u32, &mut self.buffer[FRAMING_TYPE_SIZE..]);

        // write footer
        pack_u32(
            message_size as u32,
            &mut self.buffer[self.count..(self.count + FRAMING_DATA_SIZE)],
        );

        self.finished = true;
        (message_size, message_size + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE)
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

pub struct OwnedFramedMessageWriter {
    buffer: Vec<u8>,
    count: usize,
}

impl OwnedFramedMessageWriter {
    pub fn new() -> OwnedFramedMessageWriter {
        let mut buffer = Vec::new();
        Self::push_empty_bytes(&mut buffer, FRAMING_HEADER_SIZE);
        OwnedFramedMessageWriter {
            buffer,
            count: FRAMING_HEADER_SIZE,
        }
    }

    pub fn finish(mut self) -> Result<(Vec<u8>, usize, usize), Error> {
        let message_size = self.count - FRAMING_HEADER_SIZE;

        // write header
        pack_u16(0, &mut self.buffer[0..FRAMING_TYPE_SIZE]);
        pack_u32(message_size as u32, &mut self.buffer[FRAMING_TYPE_SIZE..]);

        // write footer
        Self::push_empty_bytes(&mut self.buffer, FRAMING_FOOTER_SIZE);
        pack_u32(
            message_size as u32,
            &mut self.buffer[self.count..(self.count + FRAMING_DATA_SIZE)],
        );

        let data_size = message_size + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE;
        let buf_size = self.buffer.len();
        info!(
            "size={} data_size={} buf_size={} buffer={:?}",
            message_size, data_size, buf_size, self.buffer
        );

        Ok((self.buffer, message_size, data_size))
    }

    fn push_empty_bytes(buffer: &mut Vec<u8>, count: usize) {
        for i in 0..count {
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

fn write_framed_builder_into_buffer<A: capnp::message::Allocator>(
    buffer: &mut [u8],
    message_builder: &capnp::message::Builder<A>,
) -> std::io::Result<(usize, usize)> {
    let mut framed_rwitter = FramedMessageWriter::new(buffer);
    capnp::serialize::write_message(&mut framed_rwitter, &message_builder)?;
    let (message_size, data_size) = framed_rwitter.finish();
    Ok((message_size, data_size))
}

fn get_framed_message_size(buffer: &[u8]) -> Result<(usize, usize), Error> {
    let _message_type = unpack_u16(&buffer[0..FRAMING_TYPE_SIZE]);

    let msg_size_begin = unpack_u32(&buffer[FRAMING_TYPE_SIZE..(FRAMING_TYPE_SIZE + FRAMING_DATA_SIZE)]) as usize;
    let data_size = msg_size_begin + FRAMING_HEADER_SIZE + FRAMING_FOOTER_SIZE;
    if buffer.len() >= data_size {
        let footer_offset = FRAMING_HEADER_SIZE + msg_size_begin;
        let msg_size_end = unpack_u32(&buffer[footer_offset..(footer_offset + FRAMING_DATA_SIZE)]) as usize;
        if msg_size_begin == 0 {
            Err(Error::EOF)
        } else if msg_size_begin == msg_size_end && msg_size_begin != 0 {
            Ok((msg_size_begin, data_size))
        } else {
            warn!(
                "Invalid frame size information: begin {} != end {}",
                msg_size_begin, msg_size_end
            );
            Err(Error::InvalidData)
        }
    } else {
        warn!(
            "Invalid frame size: would exceed buffer size: msg_size_begin={} data_size={} buffer_size={}",
            msg_size_begin,
            data_size,
            buffer.len()
        );
        Err(Error::InvalidData)
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
    use chain_block_capnp::block;
    use std;
    use tempdir;

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
    fn test_write_read_block() {
        let mut data = [0u8; 1000];

        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);

        let (write_msg_size, write_data_size) =
            write_framed_builder_into_buffer(&mut data, &message_builder).unwrap();
        let framed_message = FramedSliceMessage::new(&data).unwrap();

        assert_eq!(framed_message.message_size, write_msg_size);
        assert_eq!(framed_message.data_size, write_data_size);

        let block_reader = framed_message.reader.get_root::<block::Reader>().unwrap();

        assert_eq!(block_reader.get_hash().unwrap(), "block_hash");
        assert_eq!(block_reader.get_entries().unwrap().len(), 1);
        assert_eq!(
            block_reader
                .get_entries()
                .unwrap()
                .get(0)
                .get_hash()
                .unwrap(),
            "entry_hash"
        );
    }

    #[test]
    fn test_read_invalid_block() {
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
    fn test_write_fail_not_enough_space() {
        let mut data = [0u8; 10];
        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);
        assert!(write_framed_builder_into_buffer(&mut data, &message_builder).is_err());
    }

    // TODO: move to a bench
    #[test]
    fn test_bench_write_read_sequential() {
        let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

        let nb_blocks = 10_000;

        let begin = std::time::Instant::now();
        let mut next_offset = 0;
        for i in 0..nb_blocks {
            let mut message_builder = capnp::message::Builder::new_default();
            build_test_block(&mut message_builder);
            let (written_size, data_size) =
                write_framed_builder_into_buffer(&mut mmap[next_offset..], &message_builder)
                    .unwrap();
            next_offset += data_size;
        }
        info!("Elapsed writes: {:?}", begin.elapsed());

        let begin = std::time::Instant::now();
        let mut next_offset = 0;
        for i in 0..nb_blocks {
            let framed_message = FramedSliceMessage::new(&mmap[next_offset..]).unwrap();
            let block_reader = framed_message.reader.get_root::<block::Reader>().unwrap();
            let _ = block_reader.get_hash();

            next_offset += framed_message.data_size;
        }
        info!(
            "Elapsed read individual: {:?} Size={}",
            begin.elapsed(),
            next_offset
        );

        let begin = std::time::Instant::now();
        let iterator = FramedMessageIterator::new(&mmap);
        assert_eq!(iterator.count(), nb_blocks);
        info!("Elapsed read iterator: {:?}", begin.elapsed());
    }

    #[test]
    fn test_framed_message_iterator() {
        let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

        let mut next_offset = 0;
        for i in 0..1000 {
            let mut message_builder = capnp::message::Builder::new_default();
            build_test_block(&mut message_builder);
            let (_msg_size, data_size) =
                write_framed_builder_into_buffer(&mut mmap[next_offset..], &message_builder)
                    .unwrap();
            next_offset += data_size;
        }

        // simple forward iteration
        let mut iterator = FramedMessageIterator::new(&mmap);
        let mut count = 0;
        let mut last_offset = 0;
        for message in iterator.by_ref() {
            assert!(last_offset == 0 || message.offset > last_offset);
            last_offset = message.offset;
            count += 1;
        }
        assert_eq!(iterator.last_error, None);

        // make sure we can deserialize
        let message = FramedMessageIterator::new(&mmap).take(1).last().unwrap();
        assert_eq!(message.offset, 0);
        let block_reader = message
            .framed_message
            .reader
            .get_root::<block::Reader>()
            .unwrap();
        assert_eq!(block_reader.get_hash().unwrap(), "block_hash");

        // invalid data should have an error
        let mut data = [0u8; 1000];
        pack_u16(10, &mut data);
        pack_u32(42, &mut data[FRAMING_TYPE_SIZE..]);
        let mut iterator = FramedMessageIterator::new(&data);
        assert_eq!(iterator.by_ref().count(), 0);
        assert!(iterator.last_error.is_some());
    }

    #[test]
    fn test_message_builder() {
        let mut message_builder = FramedMessageBuilder::<block::Owned>::new();
        {
            let mut root = message_builder.init_root();
            root.set_offset(1000);
        }
        let framed = message_builder.into_framed().unwrap();

        let reader = framed.get().unwrap();
        assert_eq!(reader.get_offset(), 1000);
    }

    #[test]
    fn test_owned_message() {
        let mut builder = capnp::message::Builder::new_default();
        build_test_block(&mut builder);

        let owned_message = FramedOwnedMessage::from_builder(&builder).unwrap();
        assert_eq!(owned_message.message_size(), 96);
        assert_eq!(owned_message.data_size(), 106);

        let slice_message = FramedSliceMessage::new(owned_message.data()).unwrap();
        assert_eq!(slice_message.message_size(), 96);
        assert_eq!(slice_message.data_size(), 106);
    }

    #[test]
    fn test_typed_message() {
        let mut builder = capnp::message::Builder::new_default();
        build_test_block(&mut builder);

        let owned_message = FramedOwnedMessage::from_builder(&builder).unwrap();
        let data = owned_message.data().to_vec();
        let typed_message = owned_message.into_typed::<block::Owned>();
        let reader = typed_message.get().unwrap();
        assert_eq!(reader.get_hash().unwrap(), "block_hash");

        let slice_message = FramedSliceMessage::new(&data).unwrap();
        let typed_message = slice_message.into_typed::<block::Owned>();
        let reader = typed_message.get().unwrap();
        assert_eq!(reader.get_hash().unwrap(), "block_hash");
    }

    pub fn build_test_block<A: capnp::message::Allocator>(
        message_builder: &mut capnp::message::Builder<A>,
    ) {
        let mut block = message_builder.init_root::<block::Builder>();
        block.set_hash("block_hash");
        let mut entries = block.init_entries(1);
        {
            let mut entry = entries.reborrow().get(0);
            entry.set_hash("entry_hash");
        }
    }

    fn create_test_file(size: u64) -> (tempdir::TempDir, std::fs::File, memmap::MmapMut) {
        let dir = tempdir::TempDir::new("test").unwrap();
        let segment_path = dir.path().join("segment");

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(segment_path)
            .unwrap();

        file.set_len(size).unwrap();

        let mmap = unsafe { memmap::MmapOptions::new().map_mut(&file).unwrap() };

        (dir, file, mmap)
    }
}
