use std::{collections::BinaryHeap, sync::Mutex, time::Duration};

use chrono::{DateTime, TimeZone, Utc};
use futures::{channel::oneshot, Future, FutureExt};

use crate::binding::__exocore_host_now;

/// Unix timestamp in nanoseconds.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn to_chrono_datetime(self) -> DateTime<Utc> {
        self.into()
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.as_nanos() as u64)
    }
}

impl std::ops::Sub<Timestamp> for Timestamp {
    type Output = Option<Duration>;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        self.0.checked_sub(rhs.0).map(Duration::from_nanos)
    }
}

impl From<u64> for Timestamp {
    fn from(v: u64) -> Self {
        Timestamp(v)
    }
}

impl From<Timestamp> for u64 {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        Utc.timestamp_nanos(ts.0 as i64)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Timestamp(dt.timestamp_nanos_opt().unwrap_or_default() as u64)
    }
}

/// Returns the current unix timestamp.
pub fn now() -> Timestamp {
    unsafe { Timestamp(__exocore_host_now()) }
}

/// Returns a future that will sleep for the given duration.
pub async fn sleep(duration: Duration) {
    let time = now() + duration;
    TIMERS.push(time).await;
}

lazy_static! {
    static ref TIMERS: Timers = Timers::new();
}

/// Timer containers polled at interval by the application runtime. The
/// `poll_timers` method is called when the application is being polled. Once
/// polled, the `next_timer_time` returns an optional timestamp at which the
/// next poll of the timers is expected.
///
/// Uses a binary heap to sort timers in trigger order so that the head always
/// represents the next timer to be triggered.
struct Timers {
    timers: Mutex<BinaryHeap<std::cmp::Reverse<Timer>>>,
}

/// Polls timers and trigger those that have expired.
pub(crate) fn poll_timers() {
    TIMERS.poll();
}

/// Returns the timestamp of the next timer that needs to be triggered. Returns
/// none if no timers are scheduled.
pub(crate) fn next_timer_time() -> Option<Timestamp> {
    TIMERS.next_timer()
}

impl Timers {
    fn new() -> Timers {
        Timers {
            timers: Mutex::new(BinaryHeap::new()),
        }
    }

    fn poll(&self) {
        let mut timers = self.timers.lock().unwrap();
        let now = now();

        loop {
            if let Some(peek) = timers.peek() {
                if peek.0.time > now {
                    return;
                }
            } else {
                return;
            }

            let timer = timers.pop().unwrap();
            let _ = timer.0.sender.send(());
        }
    }

    fn next_timer(&self) -> Option<Timestamp> {
        let timers = self.timers.lock().unwrap();
        timers.peek().map(|t| t.0.time)
    }

    fn push(&self, time: Timestamp) -> impl Future<Output = ()> {
        let (sender, receiver) = oneshot::channel();

        let mut timers = self.timers.lock().unwrap();
        timers.push(std::cmp::Reverse(Timer { time, sender }));

        receiver.map(|_| ())
    }
}

struct Timer {
    time: Timestamp,
    sender: oneshot::Sender<()>,
}

/// Not really Eq since 2 timers could have same trigger time. We only require
/// this for ordering.
impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let now1 = now();
        std::thread::sleep(Duration::from_millis(10));
        let now2 = now();
        let diff = now2 - now1;
        assert!(diff.unwrap().as_millis() >= 10);

        let now3 = now2 + Duration::from_secs(1);
        let diff = now3 - now2;
        assert_eq!(diff.unwrap().as_secs(), 1);
    }

    #[tokio::test]
    async fn test_timer() {
        let (sender_before, receiver_before) = oneshot::channel();
        let (sender_after, mut receiver_after) = oneshot::channel();
        tokio::spawn(async move {
            sender_before.send(now()).unwrap();
            sleep(Duration::from_millis(10)).await;
            sender_after.send(now()).unwrap();
        });

        let before_time = receiver_before.await.unwrap();

        // shouldn't have received yet since timer hasn't been triggered
        assert!(receiver_after.try_recv().unwrap().is_none());

        // wait for timer to be pushed, check make sure next poll reflects its value
        let mut next = next_timer_time();
        loop {
            if next.is_some() {
                break;
            }

            tokio::time::sleep(Duration::from_micros(100)).await;
            next = next_timer_time();
        }
        let next_dur = (next.unwrap() - before_time).unwrap();
        assert!(next_dur.as_millis() > 5);

        // wait for timer to expire, and trigger it
        tokio::time::sleep(Duration::from_millis(10)).await;
        poll_timers();

        // timer should have been triggered and 10ms or more should have elapsed
        let after_time = receiver_after.await.unwrap();
        let time_diff = (after_time - before_time).unwrap();
        assert!(time_diff.as_millis() >= 10);
    }
}
