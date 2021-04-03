use std::ops::{Bound, Range, RangeBounds};

use bytes::{Buf, Bytes};
use exocore_core::framing::FrameReader;

/// Abstraction for sources of data / bytes (in memory owned bytes, in memory
/// data reference, mmap file, etc.)
pub trait Data: FrameReader<OwnedType = Bytes> + Clone {
    fn slice<R: RangeBounds<usize>>(&self, r: R) -> &[u8];

    fn view<R: RangeBounds<usize>>(&self, r: R) -> Self;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Data from in memory owned bytes.
impl Data for Bytes {
    fn slice<R: RangeBounds<usize>>(&self, r: R) -> &[u8] {
        let r = translate_range(0, self.len(), r);
        &self.chunk()[r]
    }

    fn view<R: RangeBounds<usize>>(&self, r: R) -> Self {
        self.slice(r)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

/// Data from a referenced sliced of bytes.
#[derive(Clone)]
pub struct RefData<'s> {
    pub(crate) data: &'s [u8],
    pub(crate) start: usize,
    pub(crate) end: usize, // exclusive
}

impl<'s> RefData<'s> {
    pub fn new(data: &[u8]) -> RefData {
        RefData {
            data,
            start: 0,
            end: data.len(),
        }
    }
}

impl<'s> Data for RefData<'s> {
    fn slice<R: RangeBounds<usize>>(&self, r: R) -> &[u8] {
        let r = translate_range(self.start, self.end, r);
        &self.data[r]
    }

    fn view<R: RangeBounds<usize>>(&self, r: R) -> RefData<'s> {
        let r = translate_range(self.start, self.end, r);
        RefData {
            data: self.data,
            start: r.start,
            end: r.end,
        }
    }

    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<'s> FrameReader for RefData<'s> {
    type OwnedType = Bytes;

    fn exposed_data(&self) -> &[u8] {
        self.slice(..)
    }

    fn whole_data(&self) -> &[u8] {
        self.slice(..)
    }

    fn to_owned_frame(&self) -> Self::OwnedType {
        panic!(
            "Cannot call to_owned_frame since it could copy unbounded amount of referenced bytes"
        )
    }
}

/// Data from a memory mapped file.
#[cfg(feature = "mmap")]
pub use mmap::*;

#[cfg(feature = "mmap")]
mod mmap {
    use std::{ops::RangeBounds, sync::Arc};

    use bytes::Bytes;
    use exocore_core::framing::FrameReader;

    use super::{translate_range, Data};

    #[derive(Clone)]
    pub struct MmapData {
        pub(crate) data: Arc<memmap2::Mmap>,
        pub(crate) start: usize,
        pub(crate) end: usize, // exclusive
    }

    impl MmapData {
        pub fn from_mmap(data: Arc<memmap2::Mmap>, len: usize) -> MmapData {
            MmapData {
                data,
                start: 0,
                end: len,
            }
        }
    }

    impl Data for MmapData {
        fn slice<R: RangeBounds<usize>>(&self, r: R) -> &[u8] {
            let r = translate_range(self.start, self.end, r);
            &self.data[r]
        }

        fn view<R: RangeBounds<usize>>(&self, r: R) -> MmapData {
            let r = translate_range(self.start, self.end, r);
            MmapData {
                data: self.data.clone(),
                start: r.start,
                end: r.end,
            }
        }

        fn len(&self) -> usize {
            self.end - self.start
        }
    }

    impl FrameReader for MmapData {
        type OwnedType = Bytes;

        fn exposed_data(&self) -> &[u8] {
            self.slice(..)
        }

        fn whole_data(&self) -> &[u8] {
            self.slice(..)
        }

        fn to_owned_frame(&self) -> Self::OwnedType {
            panic!("Cannot call to_owned_frame since it could be a whole mmap file")
        }
    }
}

fn translate_range<R: RangeBounds<usize>>(start: usize, end: usize, range: R) -> Range<usize> {
    let new_start = match range.start_bound() {
        Bound::Included(s) => start + *s,
        Bound::Excluded(s) => start + *s + 1,
        Bound::Unbounded => start,
    };
    let new_end = match range.end_bound() {
        Bound::Included(s) => (start + *s + 1).min(end),
        Bound::Excluded(s) => (start + *s).min(end),
        Bound::Unbounded => end,
    };

    Range {
        start: new_start,
        end: new_end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_range() {
        assert_eq!(translate_range(0, 99, ..), Range { start: 0, end: 99 });
        assert_eq!(translate_range(2, 99, ..), Range { start: 2, end: 99 });
        assert_eq!(translate_range(2, 99, ..120), Range { start: 2, end: 99 });
        assert_eq!(translate_range(10, 99, 0..9), Range { start: 10, end: 19 });
        assert_eq!(translate_range(10, 99, 0..=9), Range { start: 10, end: 20 });
        assert_eq!(translate_range(10, 99, ..9), Range { start: 10, end: 19 });
        assert_eq!(translate_range(10, 99, ..10), Range { start: 10, end: 20 });
        assert_eq!(translate_range(10, 99, 80..), Range { start: 90, end: 99 });
    }
}
