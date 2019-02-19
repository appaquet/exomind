use std;
use std::ops::Range;

pub fn are_continuous<'a, I, T: 'a + Ord + Copy + Eq>(iter: I) -> bool
where
    I: Iterator<Item = &'a Range<T>>,
{
    get_gaps(iter).is_empty()
}

pub fn get_gaps<'a, I, T: 'a + Ord + Copy + Eq>(iter: I) -> Vec<Range<T>>
where
    I: Iterator<Item = &'a Range<T>>,
{
    let mut sorted: Vec<&Range<T>> = iter.collect();
    sorted.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut gaps = Vec::new();
    for i in 1..sorted.len() {
        let left = &sorted[i - 1];
        let right = &sorted[i];
        if left.end != right.start {
            gaps.push(Range {
                start: left.end,
                end: right.start,
            });
        }
    }

    gaps
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_gaps() {
        let a = 10..20;
        let b = 30..40;
        let c = 40..50;
        let d = 80..100;

        let ranges = vec![b, a, d, c];
        let gaps = get_gaps(ranges.iter());
        assert_eq!(gaps, vec![20..30, 50..80]);
        assert!(!are_continuous(ranges.iter()));

        let e = 30..40;
        let f = 40..50;
        let ranges = vec![e, f];
        assert!(are_continuous(ranges.iter()));
    }
}
