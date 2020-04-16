use std::collections::{BinaryHeap, LinkedList};

/// Iterator that extracts top results from underlying descend sorted iterator,
/// but taking into account a scoring function that returns an original ordering
/// and a penalized ordering.  This property allows early exit and prevent us
/// from iterating through all results to extract top results.
pub struct RescoredTopResultsIterator<I, E, F, S>
where
    I: Iterator<Item = E>,
    S: Ord,
    F: Fn(&E) -> (S, S),
{
    count: usize,
    inner_iter: Option<I>,
    scorer: F,
    top_results: LinkedList<E>,
}

impl<I, E, F, S> Iterator for RescoredTopResultsIterator<I, E, F, S>
where
    I: Iterator<Item = E>,
    S: Ord,
    F: Fn(&E) -> (S, S),
{
    type Item = E;

    fn next(&mut self) -> Option<E> {
        // on first call, we consume the inner iterator and build the top results
        if self.inner_iter.is_some() {
            // create a sorted reserve in which we will add items that are uncertain to be
            // next, but that will get added once we find an item from
            // underlying iterator whose score is smaller than reserve's
            // penalized score
            let mut reserve = BinaryHeap::<ReserveItem<E, S>>::new();

            let inner_iter = self.inner_iter.take().unwrap();
            for item in inner_iter {
                let (score, penalized_score) = (self.scorer)(&item);
                debug_assert!(
                    score >= penalized_score,
                    "penalized score can't be higher than original score"
                );

                // if we find items in reserve that have higher score than the original score of
                // current item, it means we can add these reserve items since we will never
                // find any new results with higher score
                while is_reserve_peek_higher(&mut reserve, &score)
                    && self.top_results.len() < self.count
                {
                    self.top_results.push_back(reserve.pop().unwrap().item);
                }

                if self.top_results.len() >= self.count {
                    break;
                }

                reserve.push(ReserveItem {
                    item,
                    penalized_score,
                });
            }

            // if we don't have results, but have items still in reserve, transfer them
            while self.top_results.len() < self.count && !reserve.is_empty() {
                self.top_results.push_back(reserve.pop().unwrap().item);
            }
        }

        // if we are here, inner iterator was consumed and we just dequeue from to
        // results
        self.top_results.pop_front()
    }
}

struct ReserveItem<E, S: Ord> {
    item: E,
    penalized_score: S,
}

impl<E, S: Ord> PartialOrd for ReserveItem<E, S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.penalized_score.partial_cmp(&other.penalized_score)
    }
}

impl<E, S: Ord> Ord for ReserveItem<E, S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.penalized_score.cmp(&other.penalized_score)
    }
}

impl<E, S: Ord> PartialEq for ReserveItem<E, S> {
    fn eq(&self, other: &Self) -> bool {
        self.penalized_score == other.penalized_score
    }
}

impl<E, S: Ord> Eq for ReserveItem<E, S> {}

fn is_reserve_peek_higher<E, S: Ord>(
    reserve: &mut BinaryHeap<ReserveItem<E, S>>,
    score: &S,
) -> bool {
    if let Some(first_reserve) = reserve.peek() {
        first_reserve.penalized_score >= *score
    } else {
        false
    }
}

/// Add `top_negatively_rescored_results` function to any iterator.
pub trait RescoredTopResultsIterable: Iterator {
    /// Iterator that extracts top results from underlying descend sorted
    /// iterator, but taking into account a scoring function that returns an
    /// original ordering and a penalized ordering.  This property
    /// allows early exit and prevent us from iterating through all results to
    /// extract top results.
    fn top_negatively_rescored_results<F, S>(
        self,
        count: usize,
        rescorer: F,
    ) -> RescoredTopResultsIterator<Self, Self::Item, F, S>
    where
        Self: Sized,
        S: Ord,
        F: Fn(&Self::Item) -> (S, S),
    {
        RescoredTopResultsIterator {
            count,
            inner_iter: Some(self),
            scorer: rescorer,
            top_results: LinkedList::new(),
        }
    }
}

impl<T> RescoredTopResultsIterable for T where T: Iterator {}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_rescored_top_results() {
        let data = vec![2, 1];
        let results = data
            .into_iter()
            .top_negatively_rescored_results(4, |e| (*e, *e))
            .collect_vec();
        assert_eq!(results, vec![2, 1]);

        let data = vec![10, 8, 6, 5, 4, 2, 1];
        let results = data
            .into_iter()
            .top_negatively_rescored_results(4, |e| (*e, *e))
            .collect_vec();
        assert_eq!(results, vec![10, 8, 6, 5]);

        let data = vec![20, 18, 15, 12, 10, 8, 5, 3, 2];
        let results = data
            .into_iter()
            .top_negatively_rescored_results(7, |e| match *e {
                18 => (18, 11),
                12 => (12, 4),
                o => (o, o),
            })
            .collect_vec();

        // order followed remapped
        assert_eq!(results, vec![20, 15, 18 /* 11 */, 10, 8, 5, 12 /* 4 */]);
    }
}
