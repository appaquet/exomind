pub fn timestamp_parts_to_datetime(secs: i64, nanos: i32) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(secs, nanos as u32).unwrap_or_default()
}
