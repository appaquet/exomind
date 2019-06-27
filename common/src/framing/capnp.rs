use super::{check_into_size, Error, FrameBuilder, FrameReader};
use crate::serialization::protos::MessageType;
use capnp::message::{Builder, HeapAllocator, Reader, ReaderSegments};
use capnp::traits::Owned;
use capnp::Word;
use std::io;

///
/// Frame that wraps a Capnproto message
///
pub struct CapnpFrame<I: FrameReader> {
    inner: I,
    segment_slices: Vec<(usize, usize)>,
    offset: usize,
}

impl<I: FrameReader> CapnpFrame<I> {
    pub fn new(inner: I) -> Result<CapnpFrame<I>, capnp::Error> {
        let mut data = inner.exposed_data();

        let opts = capnp::message::ReaderOptions::new();
        let (_total_words, segment_slices) = capnp::serialize::read_segment_table(&mut data, opts)?;

        // read segment reads words that we don't want to return anymore.
        // we calculate offset of cursor that we need to apply on get_segment
        let offset = inner.exposed_data().len() - data.len();

        Ok(CapnpFrame {
            inner,
            segment_slices,
            offset,
        })
    }

    pub fn inner(&self) -> &I {
        &self.inner
    }
}

impl<I: FrameReader> ReaderSegments for CapnpFrame<I> {
    fn get_segment(&self, id: u32) -> Option<&[Word]> {
        // Unsafe because of https://github.com/capnproto/capnproto-rust/issues/101
        let words = unsafe {
            let bytes = self.inner.exposed_data();
            Word::bytes_to_words(&bytes[self.offset..])
        };
        if id < self.segment_slices.len() as u32 {
            let (a, b) = self.segment_slices[id as usize];
            Some(&words[a..b])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.segment_slices.len()
    }
}

impl<I: FrameReader> FrameReader for CapnpFrame<I> {
    type OwnedType = CapnpFrame<I::OwnedType>;

    fn exposed_data(&self) -> &[u8] {
        self.inner.exposed_data()
    }

    fn whole_data(&self) -> &[u8] {
        self.inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        let owned_inner = self.inner.to_owned_frame();
        CapnpFrame::new(owned_inner).expect("Couldn't read owned version of self")
    }
}

impl<I: FrameReader + Clone> Clone for CapnpFrame<I> {
    fn clone(&self) -> Self {
        CapnpFrame {
            inner: self.inner.clone(),
            segment_slices: self.segment_slices.clone(),
            offset: self.offset,
        }
    }
}

///
/// Frame that wraps a Capnpframe with type annotation.
///
pub struct TypedCapnpFrame<I: FrameReader, T>
where
    T: for<'a> MessageType<'a>,
{
    reader: Reader<CapnpFrame<I>>,
    phantom: std::marker::PhantomData<T>,
}

impl<I: FrameReader, T> TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn new(data: I) -> Result<TypedCapnpFrame<I, T>, capnp::Error> {
        let frame = CapnpFrame::new(data)?;
        Ok(Self::from_capnp(frame))
    }

    pub fn from_capnp(capnp_frame: CapnpFrame<I>) -> TypedCapnpFrame<I, T> {
        let opts = capnp::message::ReaderOptions::new();
        let reader = Reader::new(capnp_frame, opts);

        TypedCapnpFrame {
            reader,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn inner(&self) -> &CapnpFrame<I> {
        self.reader.get_segments()
    }

    pub fn to_owned(&self) -> TypedCapnpFrame<I::OwnedType, T> {
        let inner = self.reader.get_segments();
        let inner_owned = inner.to_owned_frame();
        TypedCapnpFrame::from_capnp(inner_owned)
    }
}

impl<I: FrameReader, T> TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn get_reader(&self) -> Result<<T as Owned>::Reader, capnp::Error> {
        self.reader.get_root()
    }

    #[deprecated]
    pub fn get_typed_reader(&self) -> Result<<T as Owned>::Reader, capnp::Error> {
        self.reader.get_root()
    }
}

impl<I: FrameReader, T> FrameReader for TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    type OwnedType = TypedCapnpFrame<CapnpFrame<I::OwnedType>, T>;

    fn exposed_data(&self) -> &[u8] {
        let inner = self.reader.get_segments();
        inner.exposed_data()
    }

    fn whole_data(&self) -> &[u8] {
        let inner = self.reader.get_segments();
        inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        let inner = self.reader.get_segments();
        let owned_inner = inner.to_owned_frame();
        TypedCapnpFrame::new(owned_inner).expect("Couldn't read owned version of self")
    }
}

impl<I: FrameReader + Clone, T> Clone for TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    fn clone(&self) -> Self {
        Self::from_capnp(self.reader.get_segments().clone())
    }
}

///
/// Capnproto frame builder
///
pub struct CapnpFrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    builder: Builder<HeapAllocator>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> CapnpFrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn new() -> CapnpFrameBuilder<T> {
        let builder = Builder::new_default();
        CapnpFrameBuilder {
            builder,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn get_builder(&mut self) -> <T as capnp::traits::Owned>::Builder {
        self.builder.get_root().unwrap()
    }

    #[deprecated]
    pub fn get_builder_typed(&mut self) -> <T as capnp::traits::Owned>::Builder {
        self.builder.get_root().unwrap()
    }
}

impl<T> FrameBuilder for CapnpFrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    type OwnedFrameType = TypedCapnpFrame<Vec<u8>, T>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut buffer = Vec::new();
        capnp::serialize::write_message(&mut buffer, &self.builder)?;
        writer.write_all(&buffer)?;
        Ok(buffer.len())
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let mut buffer = Vec::new();
        capnp::serialize::write_message(&mut buffer, &self.builder)?;
        check_into_size(buffer.len(), into)?;
        into[0..buffer.len()].copy_from_slice(&buffer);
        Ok(buffer.len())
    }

    fn expected_size(&self) -> Option<usize> {
        None
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        let bytes = self.as_bytes();
        TypedCapnpFrame::new(bytes).expect("Couldn't read just-created frame")
    }
}

impl<T> Default for CapnpFrameBuilder<T>
where
    T: for<'a> MessageType<'a>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::assert_builder_equals;
    use crate::serialization::protos::data_chain_capnp::block;

    #[test]
    fn assert_typed_frame_send_sync() -> Result<(), failure::Error> {
        fn test_sync<S: Send + Sync>(_sync: S) {}

        let mut frame_builder = CapnpFrameBuilder::<block::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_depth(1234);

        let frame = TypedCapnpFrame::<_, block::Owned>::new(frame_builder.as_bytes())?;
        test_sync(frame);

        Ok(())
    }

    #[test]
    fn can_build_and_read() -> Result<(), failure::Error> {
        let mut frame_builder = CapnpFrameBuilder::<block::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_depth(1234);

        assert_builder_equals(&frame_builder)?;
        let frame_bytes = frame_builder.as_bytes();

        let capnp_frame = TypedCapnpFrame::<_, block::Owned>::new(frame_bytes)?;
        let reader = capnp_frame.get_reader()?;
        assert_eq!(1234, reader.get_depth());

        let capnp_frame_owned = capnp_frame.to_owned();
        let reader = capnp_frame_owned.get_reader()?;
        assert_eq!(1234, reader.get_depth());

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> Result<(), failure::Error> {
        let mut frame_builder = CapnpFrameBuilder::<block::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_depth(1234);

        let capnp_frame = frame_builder.as_owned_frame();
        let reader = capnp_frame.get_reader()?;
        assert_eq!(1234, reader.get_depth());

        Ok(())
    }
}
