use crate::framing;
pub use multihash::{
    BoxedMultihashDigest, Code, MultihashDigest, MultihashGeneric, Sha3_256, Sha3_512,
};

const MULTIHASH_CODE_SIZE: usize = 2;
/// Multihash digest extension.
pub trait MultihashDigestExt: MultihashDigest<Code> + Default {
    fn size() -> usize;

    fn input_signed_frame<I: framing::FrameReader>(
        &mut self,
        frame: &framing::MultihashFrame<Self, I>,
    ) {
        self.input(frame.multihash_bytes());
    }
}

impl MultihashDigestExt for Sha3_256 {
    fn size() -> usize {
        // see `Code` variants
        MULTIHASH_CODE_SIZE + 32
    }
}

impl MultihashDigestExt for Sha3_512 {
    fn size() -> usize {
        // see `Code` variants
        MULTIHASH_CODE_SIZE + 64
    }
}
