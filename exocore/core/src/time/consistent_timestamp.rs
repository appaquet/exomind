use std::{
    ops::{Add, Sub},
    time::Duration,
};

use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Timestamp that tries to be consistent and monotonic across cluster by
/// incorporating node's unique id and a per-node counter.
///
/// It's designed to have the same resolution as a nanoseconds precision
/// timestamp by encoding node id and counter in nanoseconds.
///
/// It is similar to Twitter's Snowflake (https://t.co/cLj36EQWR1) even
/// though it has not been designed out of it. As opposed to Snowflake,
/// this consistent timestamp has a per node sequence number instead of
/// per thread.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ConsistentTimestamp(pub u64);

impl ConsistentTimestamp {
    pub fn from_context(
        unix_elapsed: Duration,
        counter: u64,
        node_clock_id: u16,
    ) -> ConsistentTimestamp {
        // we shift by 1000 for milliseconds, 1000 for node id, 1000 for the counter
        let timestamp = unix_elapsed.as_secs() * 1_000 * 1_000 * 1_000
            + u64::from(unix_elapsed.subsec_millis()) * 1_000 * 1_000
            + u64::from(node_clock_id % 1_000) * 1_000
            + counter;

        ConsistentTimestamp(timestamp)
    }

    pub fn from_unix_elapsed(dur: Duration) -> ConsistentTimestamp {
        ConsistentTimestamp(dur.as_nanos() as u64)
    }

    pub fn to_datetime(self) -> DateTime<Utc> {
        Utc.timestamp_nanos(self.0 as i64)
    }

    pub fn unix_elapsed_duration(self) -> Duration {
        Duration::from_nanos(self.0)
    }
}

impl Sub<ConsistentTimestamp> for ConsistentTimestamp {
    type Output = Option<Duration>;

    fn sub(self, rhs: ConsistentTimestamp) -> Self::Output {
        self.0.checked_sub(rhs.0).map(Duration::from_nanos)
    }
}

impl Add<ConsistentTimestamp> for ConsistentTimestamp {
    type Output = ConsistentTimestamp;

    fn add(self, rhs: ConsistentTimestamp) -> Self::Output {
        ConsistentTimestamp(self.0 + rhs.0)
    }
}

impl Add<Duration> for ConsistentTimestamp {
    type Output = ConsistentTimestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        ConsistentTimestamp(self.0 + rhs.as_nanos() as u64)
    }
}

impl Sub<Duration> for ConsistentTimestamp {
    type Output = ConsistentTimestamp;

    fn sub(self, rhs: Duration) -> Self::Output {
        ConsistentTimestamp(self.0 - rhs.as_nanos() as u64)
    }
}

impl From<u64> for ConsistentTimestamp {
    fn from(value: u64) -> Self {
        ConsistentTimestamp(value)
    }
}

impl From<ConsistentTimestamp> for u64 {
    fn from(ts: ConsistentTimestamp) -> Self {
        ts.0
    }
}

impl From<DateTime<Utc>> for ConsistentTimestamp {
    fn from(date: DateTime<Utc>) -> Self {
        ConsistentTimestamp(date.timestamp_nanos_opt().unwrap_or_default() as u64)
    }
}

impl From<ConsistentTimestamp> for DateTime<Utc> {
    fn from(s: ConsistentTimestamp) -> Self {
        s.to_datetime()
    }
}

impl From<ConsistentTimestamp> for exocore_protos::prost::Timestamp {
    fn from(ts: ConsistentTimestamp) -> Self {
        let dt = ts.to_datetime();
        exocore_protos::prost::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}

impl From<exocore_protos::prost::Timestamp> for ConsistentTimestamp {
    fn from(ts: exocore_protos::prost::Timestamp) -> Self {
        exocore_protos::prost::ProstTimestampExt::to_timestamp_nanos(&ts).into()
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
        let consistent = ConsistentTimestamp::from_unix_elapsed(dur);
        let dur_after = consistent.unix_elapsed_duration();
        assert_eq!(dur, dur_after);
    }

    #[test]
    fn consistent_time_to_chrono() {
        let now: DateTime<Utc> = Utc::now();
        let consistent = ConsistentTimestamp::from(now);
        let consistent_now = consistent.to_datetime();
        assert_eq!(now.timestamp_millis(), consistent_now.timestamp_millis());

        let consistent_now2: DateTime<Utc> = consistent.into();
        assert_eq!(now.timestamp_millis(), consistent_now2.timestamp_millis());
    }

    #[test]
    fn test_consistent_time_serialize() {
        let now: DateTime<Utc> = Utc::now();
        let consistent = ConsistentTimestamp::from(now);
        let serialized = serde_json::to_string(&consistent).unwrap();
        let deserialized: ConsistentTimestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(consistent, deserialized);
    }
}
