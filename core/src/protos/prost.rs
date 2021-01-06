pub use prost::Message;
pub use prost_types::{Any, Timestamp};

use super::{Error, NamedMessage};
use crate::time::ConsistentTimestamp;

pub trait ProstTimestampExt {
    fn to_chrono_datetime(&self) -> chrono::DateTime<chrono::Utc>;

    fn to_timestamp_nanos(&self) -> u64 {
        self.to_chrono_datetime().timestamp_nanos() as u64
    }

    fn to_consistent_timestamp(&self) -> ConsistentTimestamp {
        self.to_timestamp_nanos().into()
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

impl ProstDateTimeExt for chrono::DateTime<chrono::Utc> {
    fn to_proto_timestamp(&self) -> Timestamp {
        Timestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos() as i32,
        }
    }
}

impl ProstDateTimeExt for ConsistentTimestamp {
    fn to_proto_timestamp(&self) -> Timestamp {
        self.to_datetime().to_proto_timestamp()
    }
}

pub trait ProstMessageExt {
    fn encode_to_vec(&self) -> Vec<u8>;
}

impl<M> ProstMessageExt for M
where
    M: Message,
{
    fn encode_to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).expect("Couldn't encode into Vec");
        buf
    }
}

pub trait ProstAnyPackMessageExt {
    fn pack_to_any(&self) -> Result<Any, Error>;

    fn pack_to_stepan_any(&self) -> Result<protobuf::well_known_types::Any, Error>;
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

    fn pack_to_stepan_any(&self) -> Result<protobuf::well_known_types::Any, Error> {
        let any = self.pack_to_any()?;

        Ok(protobuf::well_known_types::Any {
            type_url: any.type_url,
            value: any.value,
            ..Default::default()
        })
    }
}
