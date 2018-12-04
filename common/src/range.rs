use std;
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Range<T: Ord + Copy + Eq> {
    pub from: T,
    pub to: T,
}

impl<T: Ord + Copy + Eq> Range<T> {
    pub fn new(from: T, to: T) -> Range<T> {
        Range { from, to }
    }

    #[inline]
    pub fn is_before(&self, other: &Range<T>) -> bool {
        self.to <= other.from
    }

    #[inline]
    pub fn is_right_before(&self, other: &Range<T>) -> bool {
        self.to == other.from
    }

    #[inline]
    pub fn is_after(&self, other: &Range<T>) -> bool {
        other.to <= self.from
    }

    #[inline]
    pub fn is_right_after(&self, other: &Range<T>) -> bool {
        other.to == self.from
    }

    #[inline]
    pub fn delta(&self, other: &Range<T>) -> T
    where
        T: std::ops::Sub<Output = T> + Copy,
    {
        if self == other {
            <T as std::ops::Sub>::sub(self.from, self.from)
        } else if self.is_before(other) {
            <T as std::ops::Sub>::sub(other.from, self.to)
        } else {
            <T as std::ops::Sub>::sub(self.from, other.to)
        }
    }
}

impl<T: Ord + Copy + Eq> PartialOrd for Range<T> {
    fn partial_cmp(&self, other: &Range<T>) -> Option<Ordering> {
        if self.from == other.from && self.to == other.to {
            Some(Ordering::Equal)
        } else if self.is_before(other) {
            Some(Ordering::Less)
        } else if self.is_after(other) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl<T: Ord + Copy + Eq> Ord for Range<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self.is_before(other) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

pub fn are_continuous<'a, I, T: Ord + Copy + Eq + 'a>(iter: I) -> bool
where
    I: Iterator<Item = &'a Range<T>>,
{
    get_gaps(iter).is_empty()
}

pub fn get_gaps<'a, I, T: Ord + Copy + Eq + 'a>(iter: I) -> Vec<Range<T>>
where
    I: Iterator<Item = &'a Range<T>>,
{
    let mut sorted: Vec<&Range<T>> = iter.collect();
    sorted.sort();

    let mut gaps = Vec::new();
    for i in 1..sorted.len() {
        let left = sorted[i - 1];
        let right = sorted[i];
        if left.to != right.from {
            gaps.push(Range::new(left.to, right.from));
        }
    }

    gaps
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_before_after() {
        let a = Range::new(10, 20);
        let b = Range::new(20, 40);
        let c = Range::new(30, 40);

        assert!(a.is_before(&b));
        assert!(!a.is_after(&b));
        assert!(a.is_before(&c));
        assert!(!a.is_after(&c));
        assert!(!b.is_before(&a));
        assert!(b.is_after(&a));
        assert!(!c.is_before(&a));
        assert!(c.is_after(&a));

        assert!(!a.is_before(&a));
        assert!(!b.is_before(&b));
        assert!(!c.is_before(&c));
        assert!(!a.is_after(&a));
        assert!(!b.is_after(&b));
        assert!(!c.is_after(&c));

        assert!(a.is_right_before(&b));
        assert!(!a.is_right_after(&b));
        assert!(!b.is_right_before(&a));
        assert!(b.is_right_after(&a));
        assert!(!a.is_right_before(&c));
    }

    #[test]
    fn test_ord() {
        let a = Range::new(10, 20);
        let b = Range::new(30, 40);
        let c = Range::new(50, 60);
        let d = Range::new(60, 70);

        let mut ranges = vec![c, d, a, b];
        ranges.sort();

        assert_eq!(ranges, vec![a, b, c, d]);
    }

    #[test]
    fn test_delta() {
        let a = Range::new(10, 20);
        let b = Range::new(30, 40);
        let c = Range::new(40, 50);

        assert_eq!(a.delta(&b), 10);
        assert_eq!(b.delta(&b), 0);
        assert_eq!(a.delta(&a), 0);
        assert_eq!(b.delta(&c), 0);
    }

    #[test]
    fn test_find_gaps() {
        let a = Range::new(10, 20);
        let b = Range::new(30, 40);
        let c = Range::new(40, 50);
        let d = Range::new(80, 100);

        let ranges = vec![b, a, d, c];
        let gaps = get_gaps(ranges.iter());
        assert_eq!(gaps, vec![Range::new(20, 30), Range::new(50, 80)]);
        assert!(!are_continuous(ranges.iter()));

        let ranges = vec![b, c];
        assert!(are_continuous(ranges.iter()));
    }
}
