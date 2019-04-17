use crate::serialization::framed::SignedFrame;
pub use parity_multihash as multihash;
pub use parity_multihash::{Hash, Multihash};
use tiny_keccak;
use tiny_keccak::Keccak;

pub trait StreamHasher: std::marker::Sized {
    fn hash(&self) -> Hash;
    fn consume(&mut self, data: &[u8]);
    fn digest_into(self, buf: &mut [u8]);

    fn consume_signed_frame<F: SignedFrame>(&mut self, frame: &F) {
        let signature_data = frame
            .signature_data()
            .expect("The frame didn't have a signature");
        self.consume(signature_data);
    }

    fn into_multihash_bytes(self) -> Vec<u8> {
        let hash = self.hash();
        let hash_code = hash.code();
        assert!(hash_code < 128, "varint hash type not supported");

        // TODO: Same code as in multihash::encode(), waiting for https://github.com/multiformats/rust-multihash/pull/30
        let size = hash.size();
        let mut output: Vec<u8> = Vec::new();
        output.resize(2 + size as usize, 0);
        output[0] = hash_code as u8;
        output[1] = size;
        self.digest_into(&mut output[2..]);

        output
    }

    fn into_multihash(self) -> Multihash {
        Multihash::from_bytes(self.into_multihash_bytes())
            .expect("Couldn't create Multihash struct from just-created multihash bytes")
    }
}

pub struct Sha3Hasher {
    hash: Hash,
    hasher: Keccak,
}

impl Sha3Hasher {
    pub fn new_256() -> Sha3Hasher {
        Sha3Hasher {
            hash: Hash::SHA3256,
            hasher: Keccak::new_sha3_256(),
        }
    }

    pub fn new_512() -> Sha3Hasher {
        Sha3Hasher {
            hash: Hash::SHA3512,
            hasher: Keccak::new_sha3_512(),
        }
    }
}

impl StreamHasher for Sha3Hasher {
    fn hash(&self) -> Hash {
        self.hash
    }

    fn consume(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn digest_into(self, out: &mut [u8]) {
        self.hasher.finalize(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha3_256_streamed_vs_encode() {
        let digest_encode = multihash::encode(Hash::SHA3256, b"hello world").unwrap();

        let mut hasher = Sha3Hasher::new_256();
        hasher.consume(b"hello world");
        let digest_streamed = hasher.into_multihash();

        assert_eq!(digest_streamed, digest_encode);
    }

    #[test]
    fn sha3_512_streamed_vs_encode() {
        let digest_encode = multihash::encode(Hash::SHA3512, b"hello world").unwrap();

        let mut hasher = Sha3Hasher::new_512();
        hasher.consume(b"hello world");
        let digest_streamed = hasher.into_multihash();

        assert_eq!(digest_streamed, digest_encode);
    }
}
