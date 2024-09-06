pub use prost::{self, DecodeError, EncodeError, Message};
pub use prost_types::{Any, Timestamp};

use super::{Error, NamedMessage};

pub trait ProstTimestampExt {
    fn to_chrono_datetime(&self) -> chrono::DateTime<chrono::Utc>;

    fn to_timestamp_nanos(&self) -> u64 {
        self.to_chrono_datetime()
            .timestamp_nanos_opt()
            .unwrap_or_default() as u64
    }
}

impl ProstTimestampExt for Timestamp {
    fn to_chrono_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        crate::time::timestamp_parts_to_datetime(self.seconds, self.nanos)
    }
}

pub trait ProstDateTimeExt {
    fn to_proto_timestamp(&self) -> Timestamp;
}

impl<Tz: chrono::TimeZone> ProstDateTimeExt for chrono::DateTime<Tz> {
    fn to_proto_timestamp(&self) -> Timestamp {
        Timestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos() as i32,
        }
    }
}

pub trait ProstAnyPackMessageExt {
    fn pack_to_any(&self) -> Result<Any, Error>;

    fn pack_to_stepan_any(&self) -> Result<protobuf::well_known_types::any::Any, Error>;
}

impl<M> ProstAnyPackMessageExt for M
where
    M: Message + NamedMessage,
{
    fn pack_to_any(&self) -> Result<Any, Error> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;

        Ok(Any {
            type_url: format!("type.googleapis.com/{}", M::full_name()),
            value: buf,
        })
    }

    fn pack_to_stepan_any(&self) -> Result<protobuf::well_known_types::any::Any, Error> {
        let any = self.pack_to_any()?;

        Ok(protobuf::well_known_types::any::Any {
            type_url: any.type_url,
            value: any.value,
            ..Default::default()
        })
    }
}
