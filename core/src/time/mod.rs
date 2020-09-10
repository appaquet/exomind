mod clock;
mod consistent_timestamp;

pub use chrono::prelude::*;
pub use clock::Clock;
pub use consistent_timestamp::ConsistentTimestamp;
pub use std::time::Duration;
pub use wasm_timer::{Instant, SystemTime, UNIX_EPOCH};

pub fn timestamp_parts_to_datetime(secs: i64, nanos: i32) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp(secs, nanos as u32),
        chrono::Utc,
    )
}

pub fn timestamp_millis_to_datetime(millis: i64) -> chrono::DateTime<chrono::Utc> {
    let secs = millis / 1000;
    let nanos = (millis - secs * 1000) * 1_000_000;

    timestamp_parts_to_datetime(secs, nanos as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_parts() {
        let date = timestamp_parts_to_datetime(1599492441, 123_000_000);
        assert_eq!("2020-09-07T15:27:21.123+00:00", date.to_rfc3339());
    }

    #[test]
    fn timestamp_millis() {
        let date = timestamp_millis_to_datetime(1599492441123);
        assert_eq!("2020-09-07T15:27:21.123+00:00", date.to_rfc3339());
    }
}
