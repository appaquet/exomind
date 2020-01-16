use protobuf;
use protobuf::descriptor::{DescriptorProto, FieldDescriptorProto_Type, FileDescriptorProto};
use protobuf::{Message, ProtobufError};
use std::sync::Arc;

use crate::protos::generated::dynamic::DynamicMessage as DynamicMessageProto;
use protobuf::types::{
    ProtobufType, ProtobufTypeBool, ProtobufTypeInt32, ProtobufTypeInt64, ProtobufTypeMessage,
    ProtobufTypeString, ProtobufTypeUint32, ProtobufTypeUint64,
};
use protobuf::well_known_types::{Any, Timestamp};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::RwLock;

pub struct Registry {
    message_descriptors: RwLock<HashMap<String, Arc<ReflectMessageDescriptor>>>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            message_descriptors: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_file_descriptor(&self, file_descriptor: FileDescriptorProto) {
        let file_descriptor = Arc::new(file_descriptor);

        for msg_descriptor in file_descriptor.get_message_type() {
            let full_name = format!(
                "{}.{}",
                file_descriptor.get_package(),
                msg_descriptor.get_name()
            );
            self.register_message_descriptor(full_name, msg_descriptor.clone());
        }
    }

    pub fn register_message_descriptor(
        &self,
        full_name: String,
        msg_descriptor: DescriptorProto,
    ) -> Arc<ReflectMessageDescriptor> {
        let mut fields = Vec::new();
        for field in msg_descriptor.get_field() {
            let field_type = match field.get_field_type() {
                FieldDescriptorProto_Type::TYPE_STRING => FieldType::String,
                FieldDescriptorProto_Type::TYPE_INT32 => FieldType::Int32,
                FieldDescriptorProto_Type::TYPE_UINT32 => FieldType::Uint32,
                FieldDescriptorProto_Type::TYPE_INT64 => FieldType::Int64,
                FieldDescriptorProto_Type::TYPE_UINT64 => FieldType::Uint64,
                FieldDescriptorProto_Type::TYPE_MESSAGE => {
                    let typ = field.get_type_name().trim_start_matches('.').to_string();
                    if typ == "google.protobuf.Timestamp" {
                        FieldType::DateTime
                    } else {
                        FieldType::Message(typ)
                    }
                }
                _ => continue,
            };

            let indexed_flag =
                if let Some(indexed_value) = field.get_options().unknown_fields.get(1000) {
                    ProtobufTypeBool::get_from_unknown(indexed_value).unwrap_or(false)
                } else {
                    false
                };

            fields.push(FieldDescriptor {
                id: field.get_number() as u32,
                name: field.get_name().to_string(),
                field_type,
                indexed_flag,
            })
        }

        let descriptor = Arc::new(ReflectMessageDescriptor {
            name: full_name.clone(),
            message: msg_descriptor,
            fields,
        });

        let mut message_descriptors = self.message_descriptors.write().unwrap();
        message_descriptors.insert(full_name, descriptor.clone());

        descriptor
    }

    pub fn get_message_descriptor(
        &self,
        full_name: &str,
    ) -> Result<Arc<ReflectMessageDescriptor>, Error> {
        let message_descriptors = self.message_descriptors.read().unwrap();
        message_descriptors
            .get(full_name)
            .cloned()
            .ok_or(Error::NotInRegistry)
    }

    pub fn get_or_register_generated_descriptor<M: Message>(
        &self,
        message: &M,
    ) -> Arc<ReflectMessageDescriptor> {
        let full_name = message.descriptor().full_name();

        {
            let message_descriptors = self.message_descriptors.read().unwrap();
            if let Some(desc) = message_descriptors.get(full_name) {
                return desc.clone();
            }
        }

        self.register_message_descriptor(
            full_name.to_string(),
            message.descriptor().get_proto().clone(),
        )
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry::new()
    }
}

pub struct ReflectMessageDescriptor {
    pub name: String,
    pub fields: Vec<FieldDescriptor>,
    pub message: DescriptorProto,
}

pub struct FieldDescriptor {
    pub id: u32,
    pub name: String,
    pub field_type: FieldType,
    pub indexed_flag: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Int32,
    Uint32,
    Int64,
    Uint64,
    DateTime,
    Message(String),
}

pub enum FieldValue {
    String(String),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    DateTime(chrono::DateTime<chrono::Utc>),
    Message(Box<dyn ReflectMessage>),
}

impl FieldValue {
    pub fn as_str(&self) -> Result<&str, Error> {
        if let FieldValue::String(value) = self {
            Ok(value.as_ref())
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    pub fn as_datetime(&self) -> Result<&chrono::DateTime<chrono::Utc>, Error> {
        if let FieldValue::DateTime(value) = self {
            Ok(value)
        } else {
            Err(Error::InvalidFieldType)
        }
    }
}

impl<'s> TryFrom<&'s FieldValue> for &'s str {
    type Error = Error;

    fn try_from(value: &'s FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::String(value) => Ok(value),
            _ => Err(Error::InvalidFieldType),
        }
    }
}

pub trait ReflectMessage {
    fn descriptor(&self) -> &ReflectMessageDescriptor;

    fn full_name(&self) -> &str {
        &self.descriptor().name
    }

    fn fields(&self) -> &[FieldDescriptor] {
        self.descriptor().fields.as_ref()
    }

    fn get_field_value(&self, field: &FieldDescriptor) -> Result<FieldValue, Error>;
}

pub struct GeneratedMessage<T: Message> {
    message: T,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl<T: Message> ReflectMessage for GeneratedMessage<T> {
    fn descriptor(&self) -> &ReflectMessageDescriptor {
        self.descriptor.as_ref()
    }

    fn get_field_value(&self, field: &FieldDescriptor) -> Result<FieldValue, Error> {
        let value = self.message.descriptor().field_by_number(field.id);
        match &field.field_type {
            FieldType::String => {
                let value = value.get_str(&self.message).to_string();
                Ok(FieldValue::String(value))
            }
            FieldType::Int32 => {
                let value = value.get_i32(&self.message);
                Ok(FieldValue::Int32(value))
            }
            FieldType::Uint32 => {
                let value = value.get_u32(&self.message);
                Ok(FieldValue::Uint32(value))
            }
            FieldType::Int64 => {
                let value = value.get_i64(&self.message);
                Ok(FieldValue::Int64(value))
            }
            FieldType::Uint64 => {
                let value = value.get_u64(&self.message);
                Ok(FieldValue::Uint64(value))
            }
            FieldType::DateTime => {
                let value = value.get_message(&self.message);
                let secs = value.descriptor().field_by_number(1).get_i64(value);
                let nanos = value.descriptor().field_by_number(2).get_i32(value);
                Ok(FieldValue::DateTime(timestamp_parts_to_datetime(
                    secs, nanos,
                )))
            }
            FieldType::Message(_full_name) => Err(Error::InvalidFieldType),
        }
    }
}

pub struct DynamicMessage {
    message: DynamicMessageProto,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl ReflectMessage for DynamicMessage {
    fn descriptor(&self) -> &ReflectMessageDescriptor {
        self.descriptor.as_ref()
    }

    fn get_field_value(&self, field: &FieldDescriptor) -> Result<FieldValue, Error> {
        let value = self
            .message
            .unknown_fields
            .get(field.id)
            .ok_or(Error::NoSuchField)?;

        match field.field_type {
            FieldType::String => {
                let value =
                    ProtobufTypeString::get_from_unknown(value).ok_or(Error::NoSuchField)?;
                Ok(FieldValue::String(value))
            }
            FieldType::Int32 => {
                let value = ProtobufTypeInt32::get_from_unknown(value).ok_or(Error::NoSuchField)?;
                Ok(FieldValue::Int32(value))
            }
            FieldType::Uint32 => {
                let value =
                    ProtobufTypeUint32::get_from_unknown(value).ok_or(Error::NoSuchField)?;
                Ok(FieldValue::Uint32(value))
            }
            FieldType::Int64 => {
                let value = ProtobufTypeInt64::get_from_unknown(value).ok_or(Error::NoSuchField)?;
                Ok(FieldValue::Int64(value))
            }
            FieldType::Uint64 => {
                let value =
                    ProtobufTypeUint64::get_from_unknown(value).ok_or(Error::NoSuchField)?;
                Ok(FieldValue::Uint64(value))
            }
            FieldType::DateTime => {
                let value = ProtobufTypeMessage::<Timestamp>::get_from_unknown(value)
                    .ok_or(Error::NoSuchField)?;
                Ok(FieldValue::DateTime(timestamp_parts_to_datetime(
                    value.seconds,
                    value.nanos,
                )))
            }
            _ => Err(Error::NoSuchField),
        }
    }
}

pub fn message_to_any<M: Message>(message: &M) -> Result<Any, Error> {
    let mut any = Any::new();
    any.set_type_url(format!(
        "type.googleapis.com/{}",
        message.descriptor().full_name()
    ));
    any.set_value(message.write_to_bytes()?);
    Ok(any)
}

pub fn from_generated<T: Message>(registry: &Registry, message: T) -> GeneratedMessage<T> {
    let descriptor = registry.get_or_register_generated_descriptor(&message);

    GeneratedMessage {
        message,
        descriptor,
    }
}

pub fn from_any(registry: &Registry, any: &Any) -> Result<DynamicMessage, Error> {
    let full_name = any.type_url.replace("type.googleapis.com/", "");

    let descriptor = registry.get_message_descriptor(&full_name)?;
    let message = protobuf::parse_from_bytes::<DynamicMessageProto>(any.get_value())?;

    Ok(DynamicMessage {
        message,
        descriptor,
    })
}

pub fn from_proto_timestamp(ts: Timestamp) -> chrono::DateTime<chrono::Utc> {
    timestamp_parts_to_datetime(ts.seconds, ts.nanos)
}

pub fn timestamp_parts_to_datetime(secs: i64, nanos: i32) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp(secs, nanos as u32),
        chrono::Utc,
    )
}

pub fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
        ..Default::default()
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Message type is not in registry")]
    NotInRegistry,
    #[fail(display = "Field doesn't exist")]
    NoSuchField,
    #[fail(display = "Invalid field type")]
    InvalidFieldType,
    #[fail(display = "Field type not supported")]
    NotSupported,
    #[fail(display = "Protobuf error: {}", _0)]
    Protobuf(ProtobufError),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::Protobuf(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protos::generated::dynamic::TestDynamicMessage;
    use chrono::Utc;

    #[test]
    fn reflect_message_from_any() -> Result<(), failure::Error> {
        let registry = Registry::new();
        registry.register_file_descriptor(
            crate::protos::generated::dynamic::file_descriptor_proto().clone(),
        );

        let now = Utc::now();
        let msg = TestDynamicMessage {
            string_field: "field1".to_string(),
            datetime_field1: Some(to_proto_timestamp(now)).into(),
            ..Default::default()
        };
        let msg_any = message_to_any(&msg)?;

        let dyn_msg = from_any(&registry, &msg_any)?;
        let gen_msg = from_generated(&registry, msg);

        assert_eq!("exocore.test.TestDynamicMessage", dyn_msg.full_name());
        assert_eq!("exocore.test.TestDynamicMessage", gen_msg.full_name());

        assert_eq!(dyn_msg.fields().len(), 6);
        assert_eq!(gen_msg.fields().len(), 6);

        let field1dyn = &dyn_msg.fields()[0];
        assert!(field1dyn.indexed_flag);
        assert_eq!(dyn_msg.get_field_value(field1dyn)?.as_str()?, "field1");
        assert_eq!(gen_msg.get_field_value(field1dyn)?.as_str()?, "field1");

        let field1gen = &gen_msg.fields()[0];
        assert!(field1dyn.indexed_flag);
        assert_eq!(dyn_msg.get_field_value(field1gen)?.as_str()?, "field1");
        assert_eq!(gen_msg.get_field_value(field1gen)?.as_str()?, "field1");

        let field2 = &dyn_msg.fields()[2];
        assert!(!field2.indexed_flag);

        let field5 = &dyn_msg.fields()[5];
        assert_eq!(dyn_msg.get_field_value(field5)?.as_datetime()?, &now);
        assert_eq!(gen_msg.get_field_value(field5)?.as_datetime()?, &now);
        assert!(field5.indexed_flag);

        Ok(())
    }

    #[test]
    fn timestamp_conversion() {
        let now = Utc::now();

        let ts = to_proto_timestamp(now);
        let dt = from_proto_timestamp(ts);

        assert_eq!(dt, now);
    }
}
