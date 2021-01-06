use multihash::{Code, StatefulHasher};
pub use multihash::{Hasher, Multihash, MultihashDigest, Sha3_256, Sha3_512};

use crate::framing;

const MULTIHASH_CODE_SIZE: usize = 2;
/// Multihash digest extension.
pub trait MultihashDigestExt: StatefulHasher + Default {
    fn input_signed_frame<I: framing::FrameReader>(
        &mut self,
        frame: &framing::MultihashFrame<Self, I>,
    ) {
        self.update(frame.multihash_bytes());
    }

    fn multihash_size() -> usize {
        MULTIHASH_CODE_SIZE + usize::from(Self::size())
    }

    fn to_multihash(&self) -> Multihash;
}

impl<T> MultihashDigestExt for T
where
    T: StatefulHasher,
    Code: for<'a> From<&'a T::Digest>,
{
    fn to_multihash(&self) -> Multihash {
        let digest = self.finalize();
        Code::multihash_from_digest(&digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_multihash() {
        let stateless = Code::Sha3_256.digest(b"Hello world");

        let mut hasher = Sha3_256::default();
        hasher.update(b"Hello world");
        let stateful = hasher.to_multihash();

        assert_eq!(stateful, stateless);
    }
}
