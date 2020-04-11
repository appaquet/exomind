use super::Error;
use crate::protos::generated::exocore_test::{TestMessage, TestMessage2};
use prost::Message;
pub use prost_types::{Any, Timestamp};

pub trait ProstTimestampExt {
    fn to_chrono_datetime(&self) -> chrono::DateTime<chrono::Utc>;

    fn to_timestamp_nanos(&self) -> u64 {
        self.to_chrono_datetime().timestamp_nanos() as u64
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

pub trait ProstMessageExt {
    fn encode_to_vec(&self) -> Result<Vec<u8>, Error>;
}

impl<M> ProstMessageExt for M
where
    M: Message,
{
    fn encode_to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf)
    }
}

pub trait ProstAnyPackMessageExt {
    fn pack_to_any(&self) -> Result<Any, Error>;

    fn pack_to_stepan_any(&self) -> Result<protobuf::well_known_types::Any, Error>;
}

impl<M> ProstAnyPackMessageExt for M
where
    M: Message + PackableMessage,
{
    fn pack_to_any(&self) -> Result<Any, Error> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;

        Ok(Any {
            type_url: format!("type.googleapis.com/{}", self.full_name()),
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

pub trait PackableMessage {
    fn full_name(&self) -> &str;
}

impl PackableMessage for TestMessage {
    fn full_name(&self) -> &str {
        "exocore.test.TestMessage"
    }
}

impl PackableMessage for TestMessage2 {
    fn full_name(&self) -> &str {
        "exocore.test.TestMessage2"
    }
}
