use crate::framing;

pub use parity_multihash as multihash;
pub use parity_multihash::{Hash, Multihash};
pub use sha3::{Digest, Sha3_256, Sha3_512};

pub trait MultihashDigest: Digest + Sized {
    fn hash_type() -> Hash;

    fn multihash_output_size() -> usize {
        2 + usize::from(Self::hash_type().size())
    }

    fn input_signed_frame<I: framing::FrameReader>(
        &mut self,
        frame: &framing::MultihashFrame<Self, I>,
    ) {
        self.input(frame.multihash_bytes());
    }

    fn into_multihash_bytes(self) -> Vec<u8> {
        let hash_type = Self::hash_type();
        let hash_code = hash_type.code();
        assert!(hash_code < 128, "varint hash type not supported");

        // See `parity_multihash::encode`
        let size = hash_type.size();
        let mut output: Vec<u8> = Vec::new();
        output.resize(2 + size as usize, 0);
        output[0] = hash_code as u8;
        output[1] = size;

        let result = self.result();
        output[2..].copy_from_slice(&result);

        output
    }

    fn into_multihash(self) -> Multihash {
        Multihash::from_bytes(self.into_multihash_bytes())
            .expect("Couldn't create Multihash struct from just-created multihash bytes")
    }

    fn digest_multihash(data: &[u8]) -> Multihash {
        let mut digest = Self::new();
        digest.input(data);
        digest.into_multihash()
    }
}

impl MultihashDigest for Sha3_256 {
    fn hash_type() -> Hash {
        Hash::SHA3256
    }
}

impl MultihashDigest for Sha3_512 {
    fn hash_type() -> Hash {
        Hash::SHA3512
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha3_256_new_streamed_vs_encode() {
        let mut digest = Sha3_256::new();
        digest.input(b"hello world");
        let streamed_multihash = digest.into_multihash();

        let lib_multihash = multihash::encode(Hash::SHA3256, b"hello world").unwrap();

        let digest_multihash = Sha3_256::digest_multihash(b"hello world");

        assert_eq!(streamed_multihash, lib_multihash);
        assert_eq!(streamed_multihash, digest_multihash);
    }

    #[test]
    fn sha3_512_new_streamed_vs_encode() {
        let mut digest = Sha3_512::new();
        digest.input(b"hello world");
        let streamed_multihash = digest.into_multihash();

        let lib_multihash = multihash::encode(Hash::SHA3512, b"hello world").unwrap();

        let digest_multihash = Sha3_512::digest_multihash(b"hello world");

        assert_eq!(streamed_multihash, lib_multihash);
        assert_eq!(streamed_multihash, digest_multihash);
    }
}
