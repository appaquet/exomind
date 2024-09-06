use std::io;

use bytes::Bytes;
use multihash::Multihash;

use super::{check_from_size, check_into_size, Error, FrameBuilder, FrameReader};
use crate::sec::hash::MultihashDigestExt;

/// Check summed frame using a multihash encoded digest
pub struct MultihashFrame<const S: usize, D: MultihashDigestExt<S>, I: FrameReader> {
    inner: I,
    phantom: std::marker::PhantomData<D>,
}

impl<const S: usize, D: MultihashDigestExt<S>, I: FrameReader> MultihashFrame<S, D, I> {
    pub fn new(inner: I) -> Result<MultihashFrame<S, D, I>, Error> {
        check_from_size(D::multihash_size(), inner.exposed_data())?;
        Ok(MultihashFrame {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn verify(&self) -> Result<bool, Error> {
        let mut data_digest = D::default();
        data_digest.update(self.exposed_data());
        let data_multihash = data_digest.to_multihash();

        let frame_multihash_bytes = self.multihash_bytes();
        let frame_multihash = Multihash::<S>::from_bytes(frame_multihash_bytes)?;

        Ok(data_multihash == frame_multihash)
    }

    pub fn multihash_bytes(&self) -> &[u8] {
        let multihash_size = D::multihash_size();
        let inner_exposed_data = self.inner.exposed_data();
        &inner_exposed_data[inner_exposed_data.len() - multihash_size..]
    }
}

impl<const S: usize, D: MultihashDigestExt<S>, I: FrameReader> FrameReader
    for MultihashFrame<S, D, I>
{
    type OwnedType = MultihashFrame<S, D, I::OwnedType>;

    fn exposed_data(&self) -> &[u8] {
        let multihash_size = D::multihash_size();
        let inner_exposed_data = self.inner.exposed_data();
        &inner_exposed_data[..inner_exposed_data.len() - multihash_size]
    }

    fn whole_data(&self) -> &[u8] {
        self.inner.whole_data()
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        MultihashFrame {
            inner: self.inner.to_owned_frame(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<const S: usize, D: MultihashDigestExt<S>, I: FrameReader + Clone> Clone
    for MultihashFrame<S, D, I>
{
    fn clone(&self) -> Self {
        MultihashFrame {
            inner: self.inner.clone(),
            phantom: std::marker::PhantomData,
        }
    }
}

/// Multihash frame builder
pub struct MultihashFrameBuilder<const S: usize, D: MultihashDigestExt<S>, I: FrameBuilder> {
    inner: I,
    phantom: std::marker::PhantomData<D>,
}

impl<const S: usize, D: MultihashDigestExt<S>, I: FrameBuilder> MultihashFrameBuilder<S, D, I> {
    pub fn new(inner: I) -> MultihashFrameBuilder<S, D, I> {
        MultihashFrameBuilder {
            inner,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn inner(&mut self) -> &mut I {
        &mut self.inner
    }
}

impl<const S: usize, D: MultihashDigestExt<S>, I: FrameBuilder> FrameBuilder
    for MultihashFrameBuilder<S, D, I>
{
    type OwnedFrameType = MultihashFrame<S, D, Bytes>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        // TODO: optimize by creating a proxied writer that digests in streaming
        let mut buffer = Vec::new();
        self.inner.write_to(&mut buffer)?;
        writer.write_all(&buffer)?;

        let mut digest = D::default();
        digest.update(&buffer);
        let digest_multihash = digest.to_multihash();
        digest_multihash.write(writer)?;

        Ok(buffer.len() + D::multihash_size())
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let inner_size = self.inner.write_into(into)?;

        let mut digest = D::default();
        digest.update(&into[..inner_size]);
        let digest_multihash = digest.to_multihash();

        let total_size = inner_size + D::multihash_size();
        check_into_size(total_size, into)?;

        digest_multihash.write(&mut into[inner_size..total_size])?;

        Ok(total_size)
    }

    fn expected_size(&self) -> Option<usize> {
        self.inner
            .expected_size()
            .map(|inner_size| inner_size + D::multihash_size())
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        MultihashFrame::new(self.as_bytes()).expect("Couldn't read just-created frame")
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;
    use crate::{
        framing::assert_builder_equals,
        sec::hash::{Sha3_256, Sha3_512},
    };

    #[test]
    fn can_build_and_read_multihash() -> anyhow::Result<()> {
        let inner = Bytes::from_static(b"hello");
        let builder = MultihashFrameBuilder::<32, Sha3_256, _>::new(inner.clone());

        assert_builder_equals(&builder)?;
        let frame_bytes = builder.as_bytes();

        let reader1 = MultihashFrame::<32, Sha3_256, _>::new(&frame_bytes[..])?;
        assert_eq!(frame_bytes, reader1.whole_data());
        assert_eq!(inner, reader1.exposed_data());
        assert!(reader1.verify()?);

        let mut modified_buffer = BytesMut::from(frame_bytes.as_ref());
        modified_buffer[0..5].copy_from_slice(b"world");
        let reader2 = MultihashFrame::<32, Sha3_256, _>::new(&modified_buffer[..])?;
        assert!(!reader2.verify()?);

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let inner = Bytes::from_static(b"hello");
        let builder = MultihashFrameBuilder::<32, Sha3_256, _>::new(inner);

        let frame = builder.as_owned_frame();
        assert!(frame.verify()?);

        assert_eq!(b"hello", frame.exposed_data());

        Ok(())
    }

    #[test]
    fn different_hashes() {
        let inner = Bytes::from_static(b"hello");
        let sha3_256 = MultihashFrameBuilder::<32, Sha3_256, _>::new(inner.clone());
        let sha2_256 = MultihashFrameBuilder::<64, Sha3_512, _>::new(inner);

        assert_ne!(sha3_256.as_bytes(), sha2_256.as_bytes());
    }
}
