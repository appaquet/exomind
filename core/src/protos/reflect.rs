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
use std::{collections::HashMap, sync::Arc};

pub trait ReflectMessage: Debug {
    fn descriptor(&self) -> &ReflectMessageDescriptor;

    fn full_name(&self) -> &str {
        &self.descriptor().name
    }

    fn fields(&self) -> &HashMap<FieldId, FieldDescriptor> {
        &self.descriptor().fields
    }

    fn get_field(&self, id: FieldId) -> Option<&FieldDescriptor> {
        self.descriptor().fields.get(&id)
    }

    fn get_field_value(&self, field_id: FieldId) -> Result<FieldValue, Error>;

    fn encode(&self) -> Result<Vec<u8>, Error>;

    fn encode_to_prost_any(&self) -> Result<prost_types::Any, Error> {
        let bytes = self.encode()?;
        Ok(prost_types::Any {
            type_url: format!("type.googleapis.com/{}", self.descriptor().name),
            value: bytes,
        })
    }
}

pub trait MutableReflectMessage: ReflectMessage {
    fn clear_field_value(&mut self, field_id: FieldId) -> Result<(), Error>;
}

pub struct DynamicMessage {
    message: Empty,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl ReflectMessage for DynamicMessage {
    fn descriptor(&self) -> &ReflectMessageDescriptor {
        self.descriptor.as_ref()
    }

    fn get_field_value(&self, field_id: FieldId) -> Result<FieldValue, Error> {
        let field = self
            .get_field(field_id)
            .ok_or_else(|| Error::NoSuchField(field_id))?;
        let value = self
            .message
            .unknown_fields
            .get(field_id)
            .ok_or(Error::NoSuchField(field_id))?;

        match field.field_type {
            FieldType::String => {
                let value = ProtobufTypeString::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::String(value))
            }
            FieldType::Int32 => {
                let value = ProtobufTypeInt32::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::Int32(value))
            }
            FieldType::Uint32 => {
                let value = ProtobufTypeUint32::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::Uint32(value))
            }
            FieldType::Int64 => {
                let value = ProtobufTypeInt64::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::Int64(value))
            }
            FieldType::Uint64 => {
                let value = ProtobufTypeUint64::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::Uint64(value))
            }
            FieldType::DateTime => {
                let value = ProtobufTypeMessage::<Timestamp>::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::DateTime(
                    crate::time::timestamp_parts_to_datetime(value.seconds, value.nanos),
                ))
            }
            FieldType::Reference => {
                let ref_msg = ProtobufTypeMessage::<Empty>::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;

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
            _ => Err(Error::NoSuchField(field_id)),
        }
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let bytes = self.message.write_to_bytes()?;
        Ok(bytes)
    }
}

impl MutableReflectMessage for DynamicMessage {
    fn clear_field_value(&mut self, field_id: FieldId) -> Result<(), Error> {
        let fields = self.message.mut_unknown_fields();
        if let Some(fields) = &mut fields.fields {
            fields.remove(&field_id);
        }

        Ok(())
    }
}

impl Debug for DynamicMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DynamicMessage")
            .field("full_name", &self.descriptor.name)
            .finish()
    }
}

pub type FieldId = u32;

pub type FieldGroupId = u32;

pub struct ReflectMessageDescriptor {
    pub name: String,
    pub fields: HashMap<FieldId, FieldDescriptor>,
    pub message: DescriptorProto,
}

#[derive(Debug)]
pub struct FieldDescriptor {
    pub id: FieldId,
    pub name: String,
    pub field_type: FieldType,

    // see exocore/index/options.proto
    pub indexed_flag: bool,
    pub sorted_flag: bool,
    pub text_flag: bool,
    pub groups: Vec<FieldGroupId>,
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

pub fn from_stepan_any(registry: &Registry, any: &Any) -> Result<DynamicMessage, Error> {
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
    fn reflect_dyn_message() -> anyhow::Result<()> {
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
        let dyn_msg = from_stepan_any(&registry, &msg_any)?;

        assert_eq!("exocore.test.TestMessage", dyn_msg.full_name());
        assert!(dyn_msg.fields().len() > 10);

        let field1 = dyn_msg.get_field(1).unwrap();
        assert!(field1.text_flag);
        assert_eq!(dyn_msg.get_field_value(1)?.as_str()?, "val1");

        let field2 = dyn_msg.get_field(2).unwrap();
        assert!(!field2.text_flag);

        let field8 = dyn_msg.get_field(8).unwrap();
        assert_eq!(dyn_msg.get_field_value(8)?.as_datetime()?, &now);
        assert!(field8.indexed_flag);

        let field_value = dyn_msg.get_field_value(13)?;
        let value_ref = field_value.as_reference()?;
        assert_eq!(value_ref.entity_id, "et1");
        assert_eq!(value_ref.trait_id, "trt1");

        let field_value = dyn_msg.get_field_value(14)?;
        let value_ref = field_value.as_reference()?;
        assert_eq!(value_ref.entity_id, "et2");
        assert_eq!(value_ref.trait_id, "");

        Ok(())
    }

    #[test]
    fn clear_value_dyn_message() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let msg = TestMessage {
            string1: "val1".to_string(),
            ..Default::default()
        };

        let msg_any = msg.pack_to_stepan_any()?;
        let mut dyn_msg = from_stepan_any(&registry, &msg_any)?;

        assert!(dyn_msg.get_field_value(1).is_ok());

        dyn_msg.clear_field_value(1).unwrap();

        assert!(dyn_msg.get_field_value(1).is_err());

        Ok(())
    }

    #[test]
    fn dyn_message_encode() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let msg = TestMessage {
            string1: "val1".to_string(),
            ..Default::default()
        };

        let msg_any = msg.pack_to_stepan_any()?;
        let dyn_msg = from_stepan_any(&registry, &msg_any)?;

        let bytes = dyn_msg.encode()?;
        assert_eq!(bytes, msg_any.value);

        let prost_any = dyn_msg.encode_to_prost_any()?;
        assert_eq!(bytes, prost_any.value);

        Ok(())
    }
}
