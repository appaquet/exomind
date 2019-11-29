use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Add, Sub};
use std::time::Duration;

///
/// Timestamp that tries to be consistent and monotonic across cluster by incorporating node's unique id
/// and a per-node counter.
///
/// It's designed to have the same resolution as a nanoseconds precision timestamp by encoding node id and counter in
/// nanoseconds.
///
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ConsistentTimestamp(pub u64);

impl ConsistentTimestamp {
    pub fn from_context(
        duration: Duration,
        counter: u64,
        node_clock_id: u16,
    ) -> ConsistentTimestamp {
        // we shift by 1000 for milliseconds, 1000 for node id, 1000 for the counter
        let timestamp = duration.as_secs() * 1_000 * 1_000 * 1_000
            + u64::from(duration.subsec_millis()) * 1_000 * 1_000
            + u64::from(node_clock_id % 1_000) * 1_000
            + counter;

        ConsistentTimestamp(timestamp)
    }

    pub fn from_duration(dur: Duration) -> ConsistentTimestamp {
        ConsistentTimestamp(dur.as_nanos() as u64)
    }

    pub fn to_datetime(self) -> DateTime<Utc> {
        Utc.timestamp_nanos(self.0 as i64)
    }

    pub fn to_duration(self) -> Duration {
        Duration::from_nanos(self.0)
    }
}

impl Sub<ConsistentTimestamp> for ConsistentTimestamp {
    type Output = Option<ConsistentTimestamp>;

    fn sub(self, rhs: ConsistentTimestamp) -> Self::Output {
        self.0.checked_sub(rhs.0).map(ConsistentTimestamp)
    }
}

impl Add<ConsistentTimestamp> for ConsistentTimestamp {
    type Output = ConsistentTimestamp;

    fn add(self, rhs: ConsistentTimestamp) -> Self::Output {
        ConsistentTimestamp(self.0 + rhs.0)
    }
}

impl From<u64> for ConsistentTimestamp {
    fn from(value: u64) -> Self {
        ConsistentTimestamp(value)
    }
}

impl Into<u64> for ConsistentTimestamp {
    fn into(self) -> u64 {
        self.0
    }
}

impl Serialize for ConsistentTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for ConsistentTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Ok(ConsistentTimestamp(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_time_to_duration() {
        let dur = Duration::from_millis(3_323_123);
        let consistent = ConsistentTimestamp::from_duration(dur);
        let dur_after = consistent.to_duration();
        assert_eq!(dur, dur_after);
    }

    #[test]
    fn consistent_time_to_chrono() {
        let now: DateTime<Utc> = Utc::now();
        let consistent = ConsistentTimestamp::from(now.timestamp_nanos() as u64);
        let consistent_now = consistent.to_datetime();
        assert_eq!(now.timestamp_millis(), consistent_now.timestamp_millis());
    }
}
