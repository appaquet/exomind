use std::{collections::HashMap, convert::TryFrom, fmt::Debug, sync::Arc};

use protobuf::{
    descriptor::DescriptorProto,
    types::{
        ProtobufType, ProtobufTypeInt32, ProtobufTypeInt64, ProtobufTypeMessage,
        ProtobufTypeString, ProtobufTypeUint32, ProtobufTypeUint64,
    },
    well_known_types::{Any, Empty, Timestamp},
};
pub use protobuf::{descriptor::FileDescriptorSet, Message};

use super::{registry::Registry, Error};
use crate::generated::exocore_store::Reference;

pub trait ReflectMessage: Debug + Sized {
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

    fn encode_json(&self, registry: &Registry) -> Result<serde_json::Value, Error> {
        reflect_message_to_json(self, registry)
    }
}

fn reflect_message_to_json<M: ReflectMessage>(
    msg: &M,
    registry: &Registry,
) -> Result<serde_json::Value, Error> {
    use serde_json::Value;

    let mut values = serde_json::Map::<String, serde_json::Value>::new();
    for (id, desc) in msg.fields() {
        let name = desc.name.clone();
        match msg.get_field_value(*id) {
            Ok(value) => match value {
                FieldValue::String(v) => {
                    values.insert(name, Value::String(v));
                }
                FieldValue::Int32(v) => {
                    values.insert(name, Value::Number(v.into()));
                }
                FieldValue::Uint32(v) => {
                    values.insert(name, Value::Number(v.into()));
                }
                FieldValue::Int64(v) => {
                    values.insert(name, Value::Number(v.into()));
                }
                FieldValue::Uint64(v) => {
                    values.insert(name, Value::Number(v.into()));
                }
                FieldValue::Reference(reference) => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("entity_id".to_string(), Value::String(reference.entity_id));
                    obj.insert("trait_id".to_string(), Value::String(reference.trait_id));
                    values.insert(name, Value::Object(obj));
                }
                FieldValue::DateTime(v) => {
                    values.insert(name, Value::String(v.to_rfc3339()));
                }
                FieldValue::Message(typ, msg) => {
                    if let Ok(msg) = FieldValue::Message(typ, msg).into_message(registry) {
                        let msg_val = msg.encode_json(registry)?;
                        values.insert(name, msg_val);
                    }
                }
            },
            Err(Error::NoSuchField(_)) => {
                // field is not set
            }
            Err(other) => return Err(other),
        }
    }

    let mut obj = serde_json::Map::new();
    obj.insert(
        "type".to_string(),
        Value::String(msg.full_name().to_string()),
    );
    obj.insert("value".to_string(), Value::Object(values));

    Ok(serde_json::Value::Object(obj))
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
            .ok_or(Error::NoSuchField(field_id))?;
        let value = self
            .message
            .unknown_fields
            .get(field_id)
            .ok_or(Error::NoSuchField(field_id))?;

        match &field.field_type {
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
            FieldType::Message(typ) => {
                let ref_msg = ProtobufTypeMessage::<Empty>::get_from_unknown(value)
                    .ok_or(Error::NoSuchField(field_id))?;
                Ok(FieldValue::Message(typ.clone(), ref_msg))
            }
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

    // see exocore/store/options.proto
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
    Message(String, Empty),
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

    pub fn into_message(self, registry: &Registry) -> Result<DynamicMessage, Error> {
        if let FieldValue::Message(typ, message) = self {
            let descriptor = registry.get_message_descriptor(&typ)?;
            Ok(DynamicMessage {
                message,
                descriptor,
            })
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
    let message = Empty::parse_from_bytes(data)?;

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
    use chrono::Utc;

    use super::*;
    use crate::{
        generated::exocore_test::TestMessage,
        prost::{ProstAnyPackMessageExt, ProstDateTimeExt},
        test::TestStruct,
    };

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
            struct1: Some(TestStruct {
                string1: "str1".to_string(),
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

        let field3 = dyn_msg.get_field(3).unwrap();
        assert_eq!(
            field3.field_type,
            FieldType::Message("exocore.test.TestStruct".to_string())
        );
        let dyn_struct = dyn_msg.get_field_value(3)?.into_message(&registry)?;
        assert_eq!(dyn_struct.get_field_value(1)?.as_str()?, "str1");

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

    #[test]
    fn dyn_message_encode_json() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let date = "2022-02-25T02:11:27.793936+00:00";
        let date = chrono::DateTime::parse_from_rfc3339(date)?;
        let msg = TestMessage {
            string1: "val1".to_string(),
            int1: 1,
            date1: Some(date.to_proto_timestamp()),
            ref1: Some(Reference {
                entity_id: "et1".to_string(),
                trait_id: "trt1".to_string(),
            }),
            struct1: Some(TestStruct {
                string1: "str1".to_string(),
            }),
            ..Default::default()
        };

        let msg_any = msg.pack_to_stepan_any()?;
        let dyn_msg = from_stepan_any(&registry, &msg_any)?;

        let value = dyn_msg.encode_json(&registry)?;
        let expected = serde_json::json!({
            "type": "exocore.test.TestMessage",
            "value": {
                "string1": "val1",
                "int1": 1,
                "date1": "2022-02-25T02:11:27.793936+00:00",
                "ref1": {
                    "entity_id": "et1",
                    "trait_id": "trt1"
                },
                "struct1": {
                    "type": "exocore.test.TestStruct",
                    "value": {
                        "string1": "str1"
                    },
                },
            }
        });

        assert_eq!(value, expected);

        Ok(())
    }
}
