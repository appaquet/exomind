use super::{check_from_size, check_into_size, Error, FrameBuilder, FrameReader};
use crate::crypto::hash::MultihashDigestExt;
use std::io;

/// Checksumed frame using a multihash encoded digest
pub struct MultihashFrame<D: MultihashDigestExt, I: FrameReader> {
    inner: I,
    phantom: std::marker::PhantomData<D>,
}

impl<D: MultihashDigestExt, I: FrameReader> MultihashFrame<D, I> {
    pub fn new(inner: I) -> Result<MultihashFrame<D, I>, Error> {
        check_from_size(D::size(), inner.exposed_data())?;
        Ok(MultihashFrame {
            inner,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn verify(&self) -> Result<bool, Error> {
        let mut digest = D::default();
        digest.input(self.exposed_data());
        let digest_result = digest.result();
        let digest_output = digest_result.as_bytes();

        let inner_exposed_data = self.inner.exposed_data();
        check_from_size(digest_output.len(), inner_exposed_data)?;
        let hash_position = inner_exposed_data.len() - digest_output.len();
        let frame_hash = &inner_exposed_data[hash_position..hash_position + digest_output.len()];

        Ok(digest_output == frame_hash)
    }

    pub fn multihash_bytes(&self) -> &[u8] {
        let multihash_size = D::size();
        let inner_exposed_data = self.inner.exposed_data();
        &inner_exposed_data[inner_exposed_data.len() - multihash_size..]
    }
}

impl<D: MultihashDigestExt, I: FrameReader> FrameReader for MultihashFrame<D, I> {
    type OwnedType = MultihashFrame<D, I::OwnedType>;

    fn exposed_data(&self) -> &[u8] {
        let multihash_size = D::size();
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

impl<D: MultihashDigestExt, I: FrameReader + Clone> Clone for MultihashFrame<D, I> {
    fn clone(&self) -> Self {
        MultihashFrame {
            inner: self.inner.clone(),
            phantom: std::marker::PhantomData,
        }
    }
}

/// Multihash frame builder
pub struct MultihashFrameBuilder<D: MultihashDigestExt, I: FrameBuilder> {
    inner: I,
    phantom: std::marker::PhantomData<D>,
}

impl<D: MultihashDigestExt, I: FrameBuilder> MultihashFrameBuilder<D, I> {
    pub fn new(inner: I) -> MultihashFrameBuilder<D, I> {
        MultihashFrameBuilder {
            inner,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn inner(&mut self) -> &mut I {
        &mut self.inner
    }
}

impl<D: MultihashDigestExt, I: FrameBuilder> FrameBuilder for MultihashFrameBuilder<D, I> {
    type OwnedFrameType = MultihashFrame<D, Vec<u8>>;

    fn write_to<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        // TODO: optimize by creating a proxied writer that digests in streaming
        let mut buffer = Vec::new();
        self.inner.write_to(&mut buffer)?;
        writer.write_all(&buffer)?;

        let mut digest = D::default();
        digest.input(&buffer);
        let digest_result = digest.result();
        let digest_bytes = digest_result.as_bytes();
        writer.write_all(&digest_bytes)?;

        Ok(buffer.len() + digest_bytes.len())
    }

    fn write_into(&self, into: &mut [u8]) -> Result<usize, Error> {
        let inner_size = self.inner.write_into(into)?;

        let mut digest = D::default();
        digest.input(&into[..inner_size]);
        let digest_result = digest.result();
        let digest_bytes = digest_result.as_bytes();
        let total_size = inner_size + digest_bytes.len();

        check_into_size(total_size, into)?;
        into[inner_size..total_size].copy_from_slice(&digest_bytes);

        Ok(total_size)
    }

    fn expected_size(&self) -> Option<usize> {
        self.inner
            .expected_size()
            .map(|inner_size| inner_size + D::size())
    }

    fn as_owned_frame(&self) -> Self::OwnedFrameType {
        MultihashFrame::new(self.as_bytes()).expect("Couldn't read just-created frame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::Sha3_256;
    use crate::framing::assert_builder_equals;

    #[test]
    fn can_build_and_read_multihash() -> anyhow::Result<()> {
        let inner = b"hello".to_vec();
        let builder = MultihashFrameBuilder::<Sha3_256, _>::new(inner.clone());

        assert_builder_equals(&builder)?;
        let frame_bytes = builder.as_bytes();

        let reader1 = MultihashFrame::<Sha3_256, _>::new(&frame_bytes[..])?;
        assert_eq!(frame_bytes, reader1.whole_data());
        assert_eq!(inner, reader1.exposed_data());
        assert!(reader1.verify()?);

        let mut modified_buffer = frame_bytes.clone();
        modified_buffer[0..5].copy_from_slice(b"world");
        let reader2 = MultihashFrame::<Sha3_256, _>::new(&modified_buffer[..])?;
        assert!(!reader2.verify()?);

        Ok(())
    }

    #[test]
    fn can_build_to_owned() -> anyhow::Result<()> {
        let inner = b"hello".to_vec();
        let builder = MultihashFrameBuilder::<Sha3_256, _>::new(inner);

        let frame = builder.as_owned_frame();
        assert!(frame.verify()?);

        assert_eq!(b"hello", frame.exposed_data());

        Ok(())
    }
}
