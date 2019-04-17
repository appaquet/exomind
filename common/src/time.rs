use crate::node::Node;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(any(test, feature = "tests_utils"))]
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// TODO: This means we can't generate more than 100 consistent time per ms for now
// TODO: But will be rewritten in consistent clock logic in ticket https://github.com/appaquet/exocore/issues/6
const CONSISTENT_COUNTER_MAX: usize = 99;

#[derive(Clone)]
pub struct Clock {
    source: Source,
    consistent_counter: Arc<AtomicUsize>,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            source: Source::System,
            consistent_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn new_mocked() -> Clock {
        Clock {
            source: Source::Mocked(std::sync::Arc::new(std::sync::RwLock::new(None))),
            consistent_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn new_fixed_mocked(instant: Instant) -> Clock {
        Clock {
            source: Source::Mocked(std::sync::Arc::new(std::sync::RwLock::new(Some(instant)))),
            consistent_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[inline]
    pub fn instant(&self) -> Instant {
        match &self.source {
            Source::System => Instant::now(),
            #[cfg(any(test, feature = "tests_utils"))]
            Source::Mocked(time) => {
                let mocked_instant = time.read().expect("Couldn't acquire read lock");
                if let Some(instant) = *mocked_instant {
                    instant
                } else {
                    Instant::now()
                }
            }
        }
    }

    pub fn consistent_time(&self, _node: &Node) -> u64 {
        // TODO: To be rewritten with https://github.com/appaquet/exocore/issues/6

        let counter = loop {
            let counter = self.consistent_counter.fetch_add(1, Ordering::SeqCst);
            if counter < CONSISTENT_COUNTER_MAX {
                break counter;
            }

            // counter is higher than MAX, we try to swap it with 0.
            // if the previous value after swap wasn't equal to what we expected, it
            // means another thread swapped / increased the value, and we need to retry
            let before_swap =
                self.consistent_counter
                    .compare_and_swap(counter, 0, Ordering::SeqCst);
            if before_swap == counter {
                break 0;
            }
        };

        let elaps = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        // we shift by 1000 for milliseconds and then 100 for the counter
        elaps.as_secs() * 1_000 * 100 + u64::from(elaps.subsec_millis()) * 100 + counter as u64
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn set_fixed_instant(&self, current_time: Instant) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");
            *mocked_instant = Some(current_time);
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn add_fixed_instant_duration(&self, duration: Duration) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");
            if let Some(current_instant) = *mocked_instant {
                *mocked_instant = Some(current_instant + duration)
            }
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }

    #[cfg(any(test, feature = "tests_utils"))]
    pub fn reset_fixed_instant(&self) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");
            *mocked_instant = None;
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Clock::new()
    }
}

#[derive(Clone)]
enum Source {
    System,
    #[cfg(any(test, feature = "tests_utils"))]
    Mocked(std::sync::Arc<std::sync::RwLock<Option<Instant>>>),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_non_mocked_clock() {
        let now = Instant::now();

        let clock = Clock::new();
        let instant1 = clock.instant();
        assert!(instant1 > now);

        let instant2 = clock.instant();
        assert!(instant2 > instant1);
    }

    #[test]
    fn test_fixed_mocked_clock() {
        let mocked_clock = Clock::new_fixed_mocked(Instant::now());
        assert_eq!(mocked_clock.instant(), mocked_clock.instant());

        let new_instant = Instant::now() - Duration::from_secs(1);
        mocked_clock.set_fixed_instant(new_instant);

        assert_eq!(mocked_clock.instant(), new_instant);

        let dur_2secs = Duration::from_secs(2);
        mocked_clock.add_fixed_instant_duration(dur_2secs);
        assert_eq!(mocked_clock.instant(), new_instant + dur_2secs);
    }

    #[test]
    fn test_unfixed_mocked_clock() {
        let mocked_clock = Clock::new_mocked();
        assert_ne!(mocked_clock.instant(), mocked_clock.instant());

        let inst = Instant::now();
        mocked_clock.set_fixed_instant(inst);

        assert_eq!(mocked_clock.instant(), inst);

        mocked_clock.reset_fixed_instant();
        assert_ne!(mocked_clock.instant(), mocked_clock.instant());
    }

    #[test]
    fn test_thread_safety() {
        let now = Instant::now();

        let mocked_clock = Arc::new(Clock::new_mocked());

        let thread_clock = Arc::clone(&mocked_clock);
        thread::spawn(move || {
            thread_clock.set_fixed_instant(now);
        })
        .join()
        .unwrap();

        assert_eq!(mocked_clock.instant(), now);
    }
}
