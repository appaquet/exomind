use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::{DateTime, TimeZone, Utc};

use super::{ConsistentTimestamp, Instant, SystemTime};
use crate::cell::Node;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/6

const CONSISTENT_COUNTER_MAX: usize = 999;

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

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn new_mocked() -> Clock {
        Clock {
            source: Source::Mocked(std::sync::Arc::new(std::sync::RwLock::new(None))),
            consistent_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn new_fixed_mocked(instant: Instant) -> Clock {
        let clock = Self::new_mocked();
        clock.set_fixed_instant(instant);
        clock
    }

    #[inline]
    pub fn instant(&self) -> Instant {
        match &self.source {
            Source::System => Instant::now(),
            #[cfg(any(test, feature = "tests-utils"))]
            Source::Mocked(time) => {
                let mocked_instant = time.read().expect("Couldn't acquire read lock");
                if let Some((fixed_instant, _fixed_unix_elaps)) = *mocked_instant {
                    fixed_instant
                } else {
                    Instant::now()
                }
            }
        }
    }

    pub fn now_chrono(&self) -> DateTime<Utc> {
        let unix_elapsed = self.unix_elapsed();
        Utc.timestamp_nanos(unix_elapsed.as_nanos() as i64)
    }

    pub fn consistent_time(&self, node: &Node) -> ConsistentTimestamp {
        let counter = loop {
            let counter = self.consistent_counter.fetch_add(1, Ordering::SeqCst);
            if counter < CONSISTENT_COUNTER_MAX {
                break counter;
            }

            // unfortunately, as soon as we roll over the counter, we need to make sure that
            // we don't have the same millisecond since that would mean its not monotonic.
            // we sleep for a millisecond.
            Self::sleep_next_millisecond();

            // counter is higher than MAX, we try to swap it with 1 (so that we can return
            // 0) if the previous value after swap wasn't equal to what we
            // expected, it means another thread swapped / increased the value,
            // and we need to retry
            if self
                .consistent_counter
                .compare_exchange(counter + 1, 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                break 0; // set to 1, but can return 0 since next will return 0
            }
        };

        let unix_elapsed = self.unix_elapsed();
        ConsistentTimestamp::from_context(unix_elapsed, counter as u64, node.consistent_clock_id())
    }

    pub fn unix_elapsed(&self) -> Duration {
        match &self.source {
            Source::System => {
                let now_system = SystemTime::now();
                now_system.duration_since(wasm_timer::UNIX_EPOCH).unwrap()
            }
            #[cfg(any(test, feature = "tests-utils"))]
            Source::Mocked(time) => {
                let mocked_instant = time.read().expect("Couldn't acquire read lock");

                if let Some((_fixed_instant, fixed_unix_elaps)) = *mocked_instant {
                    fixed_unix_elaps
                } else {
                    SystemTime::now()
                        .duration_since(wasm_timer::UNIX_EPOCH)
                        .unwrap()
                }
            }
        }
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn set_fixed_instant(&self, fixed_instant: Instant) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");

            let now_system = SystemTime::now();
            let unix_elapsed = now_system.duration_since(wasm_timer::UNIX_EPOCH).unwrap();
            let now_instant = Instant::now();

            let fixed_unix_elaps = if now_instant > fixed_instant {
                unix_elapsed - (now_instant - fixed_instant)
            } else {
                unix_elapsed + (fixed_instant - now_instant)
            };

            *mocked_instant = Some((fixed_instant, fixed_unix_elaps));
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn add_fixed_instant_duration(&self, duration: super::Duration) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");
            if let Some((current_instant, unix_elapsed)) = *mocked_instant {
                *mocked_instant = Some((current_instant + duration, unix_elapsed + duration))
            }
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn reset_fixed_instant(&self) {
        if let Source::Mocked(mocked_instant) = &self.source {
            let mut mocked_instant = mocked_instant.write().expect("Couldn't acquire write lock");
            *mocked_instant = None;
        } else {
            panic!("Called set_time, but clock source is system");
        }
    }

    /// Sleeps for a millisecond. In wasm, sleep is not implemented, so we spin.
    fn sleep_next_millisecond() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        #[cfg(target_arch = "wasm32")]
        {
            let before = Instant::now();
            while before.elapsed() < Duration::from_millis(1) {}
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
    #[cfg(any(test, feature = "tests-utils"))]
    Mocked(std::sync::Arc<std::sync::RwLock<Option<(Instant, super::Duration)>>>),
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::{super::Duration, *};
    use crate::cell::LocalNode;

    #[test]
    fn non_mocked_clock() {
        let now = Instant::now();

        let clock = Clock::new();
        let instant1 = clock.instant();
        assert!(instant1 > now);

        let instant2 = clock.instant();
        assert!(instant2 > instant1);
    }

    #[test]
    fn fixed_mocked_clock() {
        let mocked_clock = Clock::new_fixed_mocked(Instant::now());
        assert_eq!(mocked_clock.instant(), mocked_clock.instant());

        let new_instant = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
        mocked_clock.set_fixed_instant(new_instant);

        assert_eq!(mocked_clock.instant(), new_instant);

        let dur_2secs = Duration::from_secs(2);
        mocked_clock.add_fixed_instant_duration(dur_2secs);
        assert_eq!(mocked_clock.instant(), new_instant + dur_2secs);
    }

    #[test]
    fn fixed_consistent_time() {
        let mocked_clock = Clock::new_fixed_mocked(Instant::now());
        let local_node = LocalNode::generate();

        let time1 = mocked_clock.consistent_time(local_node.node());
        std::thread::sleep(Duration::from_millis(10));
        let time2 = mocked_clock.consistent_time(local_node.node());
        assert_eq!(time1 + ConsistentTimestamp::from(1), time2); // time2 is +1 because of counter

        mocked_clock.reset_fixed_instant();
        let time3 = mocked_clock.consistent_time(local_node.node());
        std::thread::sleep(Duration::from_millis(10));
        let time4 = mocked_clock.consistent_time(local_node.node());

        let elaps = Duration::from_millis(10);
        assert!((time4 - time3).unwrap() > elaps);
    }

    #[test]
    fn consistent_time_collision() {
        let mocked_clock = Clock::new_fixed_mocked(Instant::now());
        let local_node = LocalNode::generate();

        let mut last_time = ConsistentTimestamp::from(0);
        for i in 0..10000 {
            let current_time = mocked_clock.consistent_time(local_node.node());
            assert_ne!(last_time, current_time, "at iteration {i}");
            last_time = current_time;
        }
    }

    #[test]
    fn fixed_future_consistent_time() {
        let mocked_clock = Clock::new_fixed_mocked(Instant::now() + Duration::from_secs(10));
        let local_node = LocalNode::generate();

        let time1 = mocked_clock.consistent_time(local_node.node());
        std::thread::sleep(Duration::from_millis(10));
        let time2 = mocked_clock.consistent_time(local_node.node());
        assert_eq!(time1 + ConsistentTimestamp::from(1), time2); // time2 is +1 because of counter

        mocked_clock.reset_fixed_instant();
        let time3 = mocked_clock.consistent_time(local_node.node());
        std::thread::sleep(Duration::from_millis(10));
        let time4 = mocked_clock.consistent_time(local_node.node());

        let elaps = Duration::from_millis(10);
        assert!((time4 - time3).unwrap() > elaps);
    }

    #[test]
    fn unfixed_mocked_clock() {
        let mocked_clock = Clock::new_mocked();
        assert_ne!(mocked_clock.instant(), mocked_clock.instant());

        let inst = Instant::now();
        mocked_clock.set_fixed_instant(inst);

        assert_eq!(mocked_clock.instant(), inst);

        {
            // unfixed clock should now give different instant
            mocked_clock.reset_fixed_instant();

            let t1 = mocked_clock.instant();

            // ony *nix, two call will give diff instant because of sys calls, but on
            // windows it may be the same
            std::thread::sleep(Duration::from_millis(1));

            let t2 = mocked_clock.instant();

            assert_ne!(t1, t2);
        }
    }

    #[test]
    fn thread_safety() {
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

    #[test]
    fn chrono_datetime() {
        let clock = Clock::new();

        let now1 = clock.now_chrono();
        thread::sleep(Duration::from_nanos(1));
        let now2 = clock.now_chrono();
        assert!(now2 > now1);

        let fixed_clock = Clock::new_fixed_mocked(Instant::now());
        let now3 = fixed_clock.now_chrono();
        assert!(now3 > now2);

        let now4 = fixed_clock.now_chrono();
        assert_eq!(now3, now4);
    }
}
