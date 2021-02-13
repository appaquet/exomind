use chrono::{DateTime, Utc};
use protobuf::well_known_types::{Any, Timestamp};
use protobuf::{Message, SingularPtrField};

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

    fn to_proto_timestamp_ptr(&self) -> SingularPtrField<Timestamp> {
        Some(self.to_proto_timestamp()).into()
    }
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

    fn pack_to_any_ptr(&self) -> Result<SingularPtrField<Any>, Error> {
        Ok(Some(self.pack_to_any()?).into())
    }
}

impl<M> StepanMessageExt for M
where
    M: Message,
{
    fn pack_to_any(&self) -> Result<Any, Error> {
        let mut any = Any::new();
        any.set_type_url(format!(
            "type.googleapis.com/{}",
            self.descriptor().full_name()
        ));
        any.set_value(self.write_to_bytes()?);
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
