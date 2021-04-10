use std::ops::RangeBounds;

use bytes::Bytes;
use exocore_core::framing::FrameReader;

use crate::data::Data;

/// Data from the chain that can be from a mmap file or owned bytes.
#[derive(Clone)]
pub enum ChainData {
    #[cfg(feature = "mmap")]
    Mmap(crate::data::MmapData),
    Bytes(Bytes),
}

impl Data for ChainData {
    fn slice<R: RangeBounds<usize>>(&self, r: R) -> &[u8] {
        match self {
            #[cfg(feature = "mmap")]
            ChainData::Mmap(m) => Data::slice(m, r),
            ChainData::Bytes(m) => Data::slice(m, r),
        }
    }

    fn view<R: RangeBounds<usize>>(&self, r: R) -> ChainData {
        match self {
            #[cfg(feature = "mmap")]
            ChainData::Mmap(m) => ChainData::Mmap(Data::view(m, r)),
            ChainData::Bytes(m) => ChainData::Bytes(Data::view(m, r)),
        }
    }

    fn len(&self) -> usize {
        match self {
            #[cfg(feature = "mmap")]
            ChainData::Mmap(m) => Data::len(m),
            ChainData::Bytes(m) => Data::len(m),
        }
    }
}

impl FrameReader for ChainData {
    type OwnedType = Bytes;

    fn exposed_data(&self) -> &[u8] {
        self.slice(..)
    }

    fn whole_data(&self) -> &[u8] {
        self.slice(..)
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        panic!("Cannot call to_owned_frame since it could be from an unbounded source")
    }
}
