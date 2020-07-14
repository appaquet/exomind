use super::registry::Registry;
use super::Error;
use crate::protos::generated::exocore_index::Reference;
use protobuf::descriptor::DescriptorProto;
use protobuf::types::{
    ProtobufType, ProtobufTypeInt32, ProtobufTypeInt64, ProtobufTypeMessage, ProtobufTypeString,
    ProtobufTypeUint32, ProtobufTypeUint64,
};
use protobuf::well_known_types::{Any, Empty, Timestamp};
use protobuf::Message;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::Arc;

pub trait ReflectMessage: Debug {
    fn descriptor(&self) -> &ReflectMessageDescriptor;

    fn full_name(&self) -> &str {
        &self.descriptor().name
    }

    fn fields(&self) -> &[FieldDescriptor] {
        self.descriptor().fields.as_ref()
    }

    fn get_field_value(&self, field: &FieldDescriptor) -> Result<FieldValue, Error>;
}

pub struct StepanMessage<T: Message> {
    message: T,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl<T: Message> ReflectMessage for StepanMessage<T> {
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
                Ok(FieldValue::DateTime(
                    crate::time::timestamp_parts_to_datetime(secs, nanos),
                ))
            }
            FieldType::Reference => {
                let value = value.get_message(&self.message);
                let entity_id = value
                    .descriptor()
                    .field_by_number(1)
                    .get_str(value)
                    .to_string();
                let trait_id = value
                    .descriptor()
                    .field_by_number(2)
                    .get_str(value)
                    .to_string();
                Ok(FieldValue::Reference(Reference {
                    entity_id,
                    trait_id,
                }))
            }
            FieldType::Message(_full_name) => Err(Error::InvalidFieldType),
        }
    }
}

impl<T: Message> Debug for StepanMessage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("StepanMessage")
            .field("full_name", &self.descriptor.name)
            .finish()
    }
}

pub struct DynamicMessage {
    message: Empty,
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
                Ok(FieldValue::DateTime(
                    crate::time::timestamp_parts_to_datetime(value.seconds, value.nanos),
                ))
            }
            FieldType::Reference => {
                let ref_msg = ProtobufTypeMessage::<Empty>::get_from_unknown(value)
                    .ok_or(Error::NoSuchField)?;

                let entity_id_value = ref_msg
                    .unknown_fields
                    .get(1)
                    .and_then(ProtobufTypeString::get_from_unknown);
                let entity_id = entity_id_value.unwrap_or_default();

                let trait_id_value = ref_msg
                    .unknown_fields
                    .get(2)
                    .and_then(ProtobufTypeString::get_from_unknown);
                let trait_id = trait_id_value.unwrap_or_default();

                Ok(FieldValue::Reference(Reference {
                    entity_id,
                    trait_id,
                }))
            }
            _ => Err(Error::NoSuchField),
        }
    }
}

impl Debug for DynamicMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DynamicMessage")
            .field("full_name", &self.descriptor.name)
            .finish()
    }
}

pub struct ReflectMessageDescriptor {
    pub name: String,
    pub fields: Vec<FieldDescriptor>,
    pub message: DescriptorProto,
}

#[derive(Debug)]
pub struct FieldDescriptor {
    pub id: u32,
    pub name: String,
    pub field_type: FieldType,
    pub indexed_flag: bool,
    pub sorted_flag: bool,
    pub text_flag: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Int32,
    Uint32,
    Int64,
    Uint64,
    DateTime,
    Reference,
    Message(String),
}

#[derive(Debug)]
pub enum FieldValue {
    String(String),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    Reference(Reference),
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

    pub fn as_reference(&self) -> Result<&Reference, Error> {
        if let FieldValue::Reference(value) = self {
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

pub fn from_generated<T: Message>(registry: &Registry, message: T) -> StepanMessage<T> {
    let descriptor = registry.get_or_register_generated_descriptor(&message);

    StepanMessage {
        message,
        descriptor,
    }
}

pub fn from_any(registry: &Registry, any: &Any) -> Result<DynamicMessage, Error> {
    from_any_url_and_data(registry, &any.type_url, &any.value)
}

pub fn from_prost_any(
    registry: &Registry,
    any: &prost_types::Any,
) -> Result<DynamicMessage, Error> {
    from_any_url_and_data(registry, &any.type_url, &any.value)
}

pub fn from_any_url_and_data(
    registry: &Registry,
    url: &str,
    data: &[u8],
) -> Result<DynamicMessage, Error> {
    let full_name = any_url_to_full_name(url);

    let descriptor = registry.get_message_descriptor(&full_name)?;
    let message = protobuf::parse_from_bytes::<Empty>(data)?;

    Ok(DynamicMessage {
        message,
        descriptor,
    })
}

pub fn any_url_to_full_name(url: &str) -> String {
    url.replace("type.googleapis.com/", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protos::generated::exocore_test::TestMessage;
    use crate::protos::prost::{ProstAnyPackMessageExt, ProstDateTimeExt};
    use chrono::Utc;

    #[test]
    fn reflect_message_from_any() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let now = Utc::now();
        let msg = TestMessage {
            string1: "val1".to_string(),
            date1: Some(now.to_proto_timestamp()),
            ref1: Some(Reference {
                entity_id: "et1".to_string(),
                trait_id: "trt1".to_string(),
            }),
            ref2: Some(Reference {
                entity_id: "et2".to_string(),
                trait_id: String::new(),
            }),
            ..Default::default()
        };
        let msg_any = msg.pack_to_stepan_any()?;

        let dyn_msg = from_any(&registry, &msg_any)?;

        assert_eq!("exocore.test.TestMessage", dyn_msg.full_name());

        assert!(dyn_msg.fields().len() > 10);

        let field1 = &dyn_msg.fields().iter().find(|f| f.id == 1).unwrap();
        assert!(field1.text_flag);
        assert_eq!(dyn_msg.get_field_value(field1)?.as_str()?, "val1");

        let field2 = &dyn_msg.fields().iter().find(|f| f.id == 2).unwrap();
        assert!(!field2.text_flag);

        let field8 = &dyn_msg.fields().iter().find(|f| f.id == 8).unwrap();
        assert_eq!(dyn_msg.get_field_value(field8)?.as_datetime()?, &now);
        assert!(field8.indexed_flag);

        let field13 = &dyn_msg.fields().iter().find(|f| f.id == 13).unwrap();
        let field_value = dyn_msg.get_field_value(field13)?;
        let value_ref = field_value.as_reference()?;
        assert_eq!(value_ref.entity_id, "et1");
        assert_eq!(value_ref.trait_id, "trt1");

        let field14 = &dyn_msg.fields().iter().find(|f| f.id == 14).unwrap();
        let field_value = dyn_msg.get_field_value(field14)?;
        let value_ref = field_value.as_reference()?;
        assert_eq!(value_ref.entity_id, "et2");
        assert_eq!(value_ref.trait_id, "");

        Ok(())
    }
}
