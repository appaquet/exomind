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
