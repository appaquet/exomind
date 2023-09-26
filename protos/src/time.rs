pub fn timestamp_parts_to_datetime(secs: i64, nanos: i32) -> chrono::DateTime<chrono::Utc> {
    let dt = chrono::NaiveDateTime::from_timestamp_opt(secs, nanos as u32);
    chrono::DateTime::from_naive_utc_and_offset(dt.unwrap_or_default(), chrono::Utc)
}
