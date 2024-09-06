use chrono::{DateTime, Utc};
use protobuf::{
    well_known_types::{any::Any, timestamp::Timestamp},
    MessageFull,
};

use super::Error;

pub trait StepanTimestampExt {
    fn to_chrono_datetime(&self) -> chrono::DateTime<chrono::Utc>;
}

impl StepanTimestampExt for Timestamp {
    fn to_chrono_datetime(&self) -> DateTime<Utc> {
        crate::time::timestamp_parts_to_datetime(self.seconds, self.nanos)
    }
}

pub trait StepanDateTimeExt {
    fn to_proto_timestamp(&self) -> Timestamp;
}

impl StepanDateTimeExt for chrono::DateTime<Utc> {
    fn to_proto_timestamp(&self) -> Timestamp {
        Timestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos() as i32,
            ..Default::default()
        }
    }
}

pub trait StepanMessageExt {
    fn pack_to_any(&self) -> Result<Any, Error>;
}

impl<M> StepanMessageExt for M
where
    M: MessageFull,
{
    fn pack_to_any(&self) -> Result<Any, Error> {
        let mut any = Any::new();
        any.type_url = format!("type.googleapis.com/{}", M::descriptor().name(),);
        any.value = self.write_to_bytes()?;
        Ok(any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_conversion() {
        let now = Utc::now();

        let ts = now.to_proto_timestamp();
        let dt = ts.to_chrono_datetime();

        assert_eq!(dt, now);
    }
}
