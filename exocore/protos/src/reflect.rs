use std::{collections::HashMap, convert::TryFrom, fmt::Debug, sync::Arc};

pub use protobuf::{descriptor::FileDescriptorSet, Message};
use protobuf::{
    reflect::{FieldDescriptor as FieldDescriptorProto, ReflectFieldRef, ReflectValueRef},
    well_known_types::any::Any,
    MessageDyn,
};

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
        message_to_json(self, registry)
    }
}

fn message_to_json<M: ReflectMessage>(
    msg: &M,
    registry: &Registry,
) -> Result<serde_json::Value, Error> {
    use serde_json::Value;

    let mut values = serde_json::Map::<String, serde_json::Value>::new();
    for (id, desc) in msg.fields() {
        let name = desc.name.clone();
        match msg.get_field_value(*id) {
            Ok(value) => {
                let json_value = field_value_to_json(value, registry)?;
                values.insert(name, json_value);
            }
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

fn field_value_to_json(value: FieldValue, registry: &Registry) -> Result<serde_json::Value, Error> {
    use serde_json::Value;
    Ok(match value {
        FieldValue::String(v) => Value::String(v),
        FieldValue::Int32(v) => Value::Number(v.into()),
        FieldValue::Uint32(v) => Value::Number(v.into()),
        FieldValue::Int64(v) => Value::Number(v.into()),
        FieldValue::Uint64(v) => Value::Number(v.into()),
        FieldValue::Reference(reference) => {
            let mut obj = serde_json::Map::new();
            obj.insert("entity_id".to_string(), Value::String(reference.entity_id));
            obj.insert("trait_id".to_string(), Value::String(reference.trait_id));
            Value::Object(obj)
        }
        FieldValue::DateTime(v) => Value::String(v.to_rfc3339()),
        FieldValue::Message(typ, msg) => {
            let msg = FieldValue::Message(typ, msg).into_message(registry)?;
            msg.encode_json(registry)?
        }
        FieldValue::Repeated(values) => {
            let arr = values
                .into_iter()
                .map(|v| field_value_to_json(v, registry))
                .collect::<Result<Vec<_>, Error>>()?;
            Value::Array(arr)
        }
    })
}

pub trait MutableReflectMessage: ReflectMessage {
    fn clear_field_value(&mut self, field_id: FieldId) -> Result<(), Error>;
}

pub struct DynamicMessage {
    message: Box<dyn MessageDyn>,
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

        let reflect_field = field.descriptor.get_reflect(self.message.as_ref());
        convert_field_ref(field_id, &field.field_type, reflect_field)
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let bytes = self.message.write_to_bytes_dyn()?;
        Ok(bytes)
    }
}

fn convert_field_ref(
    field_id: FieldId,
    field_type: &FieldType,
    field_ref: ReflectFieldRef,
) -> Result<FieldValue, Error> {
    match field_ref {
        ReflectFieldRef::Optional(v) => match v.value() {
            Some(v) => convert_field_value(field_type, v),
            None => Err(Error::NoSuchField(field_id)),
        },
        ReflectFieldRef::Repeated(r) => {
            let FieldType::Repeated(inner_field_type) = field_type else {
                return Err(Error::Other(anyhow!(
                    "expected repeated field type, got {field_type:?} at field {field_id:?}"
                )));
            };

            let mut values = Vec::new();
            for i in 0..r.len() {
                values.push(convert_field_value(inner_field_type, r.get(i))?);
            }
            Ok(FieldValue::Repeated(values))
        }
        ReflectFieldRef::Map(_) => {
            // TODO: Implement me
            Err(Error::NoSuchField(field_id))
        }
    }
}

fn convert_field_value(
    field_type: &FieldType,
    value: ReflectValueRef,
) -> Result<FieldValue, Error> {
    match field_type {
        FieldType::String => match value {
            ReflectValueRef::String(v) => Ok(FieldValue::String(v.to_string())),
            v => Err(Error::Other(anyhow!("expected string field, got: {v:?}"))),
        },
        FieldType::Int32 => match value {
            ReflectValueRef::I32(v) => Ok(FieldValue::Int32(v)),
            v => Err(Error::Other(anyhow!("expected int32 field, got: {v:?}"))),
        },
        FieldType::Uint32 => match value {
            ReflectValueRef::U32(v) => Ok(FieldValue::Uint32(v)),
            v => Err(Error::Other(anyhow!("expected uint32 field, got: {v:?}"))),
        },
        FieldType::Int64 => match value {
            ReflectValueRef::I64(v) => Ok(FieldValue::Int64(v)),
            v => Err(Error::Other(anyhow!("expected int64 field, got: {v:?}"))),
        },
        FieldType::Uint64 => match value {
            ReflectValueRef::U64(v) => Ok(FieldValue::Uint64(v)),
            v => Err(Error::Other(anyhow!("expected uint64 field, got: {v:?}"))),
        },
        FieldType::DateTime => match value {
            ReflectValueRef::Message(msg) => {
                let msg_desc = msg.descriptor_dyn();
                let secs_desc = msg_desc.field_by_number(1).unwrap();
                let secs = secs_desc
                    .get_singular(&*msg)
                    .and_then(|v| v.to_i64())
                    .unwrap_or_default();

                let nanos_desc = msg_desc.field_by_number(2).unwrap();
                let nanos = nanos_desc
                    .get_singular(&*msg)
                    .and_then(|v| v.to_i32())
                    .unwrap_or_default();

                Ok(FieldValue::DateTime(
                    crate::time::timestamp_parts_to_datetime(secs, nanos),
                ))
            }
            v => Err(Error::Other(anyhow!(
                "expected message as timestamp field, got: {v:?}"
            ))),
        },
        FieldType::Reference => match value {
            ReflectValueRef::Message(msg) => {
                let msg_desc = msg.descriptor_dyn();
                let et_desc = msg_desc.field_by_number(1).unwrap();
                let entity_id = et_desc
                    .get_singular(&*msg)
                    .and_then(|v| v.to_str().map(|v| v.to_string()))
                    .unwrap_or_default();

                let trt_desc = msg_desc.field_by_number(2).unwrap();
                let trait_id = trt_desc
                    .get_singular(&*msg)
                    .and_then(|v| v.to_str().map(|v| v.to_string()))
                    .unwrap_or_default();

                Ok(FieldValue::Reference(Reference {
                    entity_id,
                    trait_id,
                }))
            }
            v => Err(Error::Other(anyhow!(
                "expected message as reference field, got: {v:?}"
            ))),
        },
        FieldType::Message(msg_type) => match value {
            ReflectValueRef::Message(msg) => {
                let dyn_msg = msg.clone_box();
                Ok(FieldValue::Message(msg_type.clone(), dyn_msg))
            }
            v => Err(Error::Other(anyhow!(
                "expected field to be a message, got: {v:?}"
            ))),
        },
        FieldType::Repeated(_) => {
            unreachable!("repeated fields should have been handled in convert_field_ref");
        }
    }
}

impl MutableReflectMessage for DynamicMessage {
    fn clear_field_value(&mut self, field_id: FieldId) -> Result<(), Error> {
        let field = self
            .descriptor
            .fields
            .get(&field_id)
            .ok_or(Error::NoSuchField(field_id))?;

        if !field.descriptor.has_field(self.message.as_ref()) {
            return Ok(());
        }

        if field.descriptor.is_repeated() {
            let mut repeated = field.descriptor.mut_repeated(self.message.as_mut());
            repeated.clear();
        } else if field.descriptor.is_map() {
            let mut map = field.descriptor.mut_map(self.message.as_mut());
            map.clear();
        } else {
            field.descriptor.clear_field(self.message.as_mut());
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
    pub name: String, // full name of the message
    pub fields: HashMap<FieldId, FieldDescriptor>,
    pub message: protobuf::reflect::MessageDescriptor,

    // see exocore/store/options.proto
    pub short_names: Vec<String>,
}

pub struct FieldDescriptor {
    pub id: FieldId,
    pub descriptor: FieldDescriptorProto,
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
    Repeated(Box<FieldType>),
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
    Message(String, Box<dyn MessageDyn>),
    Repeated(Vec<FieldValue>),
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
    let message = descriptor.message.parse_from_bytes(data)?;

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

        let mut map1 = HashMap::new();
        map1.insert("key1".to_string(), "value1".to_string());
        map1.insert("key2".to_string(), "value2".to_string());

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
            map1,
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

        // TODO: Maps not supported yet
        // let field22 = dyn_msg.get_field(22).unwrap();
        // assert_eq!(
        //     field22.field_type,
        //     FieldType::Repeated(Box::new(FieldType::Message(
        //         "exocore.test.TestMessage.Map1Entry".to_string()
        //     )))
        // );
        // let _field_value = dyn_msg.get_field_value(22)?;

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
