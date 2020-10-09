use super::{check_into_size, Error, FrameBuilder, FrameReader};
use crate::protos::generated::MessageType;
use capnp::message::{Builder, HeapAllocator};
use capnp::traits::Owned;
use std::io;

/// Frame that wraps a Capnproto message
pub struct CapnpFrame<I: FrameReader> {
    inner: I,
}

impl<I: FrameReader> CapnpFrame<I> {
    pub fn new(inner: I) -> Result<CapnpFrame<I>, capnp::Error> {
        Ok(CapnpFrame { inner })
    }

    pub fn inner(&self) -> &I {
        &self.inner
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
        }
    }
}

/// Frame that wraps a Capnpframe with type annotation.
pub struct TypedCapnpFrame<I: FrameReader, T>
where
    T: for<'a> MessageType<'a>,
{
    inner: CapnpFrame<I>,
    reader: capnp::message::Reader<capnp::serialize::OwnedSegments>,
    phantom: std::marker::PhantomData<T>,
}

impl<I: FrameReader, T> TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    pub fn new(data: I) -> Result<TypedCapnpFrame<I, T>, capnp::Error> {
        let frame = CapnpFrame::new(data)?;
        Self::from_capnp(frame)
    }

    pub fn from_capnp(capnp_frame: CapnpFrame<I>) -> Result<TypedCapnpFrame<I, T>, capnp::Error> {
        let opts = capnp::message::ReaderOptions {
            // This remove security limit, but we keep reusing the reader and we eventually reach
            // that limit because of it.
            traversal_limit_in_words: std::usize::MAX as u64,
            ..Default::default()
        };
        let mut data = capnp_frame.exposed_data();
        let reader = capnp::serialize::read_message(&mut data, opts)?;

        Ok(TypedCapnpFrame {
            inner: capnp_frame,
            reader,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn inner(&self) -> &CapnpFrame<I> {
        &self.inner
    }

    pub fn get_reader(&self) -> Result<<T as Owned>::Reader, capnp::Error> {
        self.reader.get_root()
    }

    pub fn to_owned(&self) -> TypedCapnpFrame<I::OwnedType, T> {
        let inner_owned = self.inner.to_owned_frame();
        TypedCapnpFrame::from_capnp(inner_owned).unwrap()
    }
}

impl<I: FrameReader, T> FrameReader for TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    type OwnedType = TypedCapnpFrame<CapnpFrame<I::OwnedType>, T>;

    fn exposed_data(&self) -> &[u8] {
        self.inner.exposed_data()
    }

    fn whole_data(&self) -> &[u8] {
        self.inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        let owned_inner = self.inner.to_owned_frame();
        TypedCapnpFrame::new(owned_inner).expect("Couldn't read owned version of self")
    }
}

impl<I: FrameReader + Clone, T> Clone for TypedCapnpFrame<I, T>
where
    T: for<'a> MessageType<'a>,
{
    fn clone(&self) -> Self {
        Self::from_capnp(self.inner.clone()).unwrap()
    }
}

/// Capnproto frame builder
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
    use crate::protos::generated::data_chain_capnp::block_header;

    #[test]
    fn assert_typed_frame_send_sync() -> anyhow::Result<()> {
        fn test_sync<S: Send + Sync>(_sync: S) {}

        let mut frame_builder = CapnpFrameBuilder::<block_header::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_height(1234);

        let frame = TypedCapnpFrame::<_, block_header::Owned>::new(frame_builder.as_bytes())?;
        test_sync(frame);

        Ok(())
    }

    #[test]
    fn can_build_and_read() -> anyhow::Result<()> {
        let mut frame_builder = CapnpFrameBuilder::<block_header::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_height(1234);

        assert_builder_equals(&frame_builder)?;
        let frame_bytes = frame_builder.as_bytes();

        let capnp_frame = TypedCapnpFrame::<_, block_header::Owned>::new(frame_bytes)?;
        let reader = capnp_frame.get_reader()?;
        assert_eq!(1234, reader.get_height());

        let capnp_frame_owned = capnp_frame.to_owned();
        let reader = capnp_frame_owned.get_reader()?;
        assert_eq!(1234, reader.get_height());

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let mut frame_builder = CapnpFrameBuilder::<block_header::Owned>::new();
        let mut builder = frame_builder.get_builder();
        builder.set_height(1234);

        let capnp_frame = frame_builder.as_owned_frame();
        let reader = capnp_frame.get_reader()?;
        assert_eq!(1234, reader.get_height());

        Ok(())
    }
}
