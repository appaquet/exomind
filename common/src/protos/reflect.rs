use protobuf;
use protobuf::descriptor::{DescriptorProto, FieldDescriptorProto_Type, FileDescriptorProto};
use protobuf::{Message, ProtobufError};
use std::sync::Arc;

use crate::protos::generated::dynamic::DynamicMessage as DynamicMessageProto;
use protobuf::types::{ProtobufType, ProtobufTypeBool, ProtobufTypeString};
use protobuf::well_known_types::Any;
use std::collections::HashMap;
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
        let file_descriptor = Arc::new(file_descriptor.clone());

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
    Message(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    String(String),
}

pub trait ReflectMessage {
    type Message: Message;

    fn descriptor(&self) -> &ReflectMessageDescriptor;

    fn inner(&self) -> &Self::Message;

    fn full_name(&self) -> &str {
        &self.descriptor().name
    }

    fn fields(&self) -> &[FieldDescriptor] {
        self.descriptor().fields.as_ref()
    }

    fn field_value(&self, field: &FieldDescriptor) -> Option<FieldValue>;
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

pub struct GeneratedMessage<T: Message> {
    message: T,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl<T: Message> ReflectMessage for GeneratedMessage<T> {
    type Message = T;

    fn descriptor(&self) -> &ReflectMessageDescriptor {
        self.descriptor.as_ref()
    }

    fn inner(&self) -> &T {
        &self.message
    }

    fn field_value(&self, field: &FieldDescriptor) -> Option<FieldValue> {
        let value = self.message.descriptor().field_by_number(field.id);
        match field.field_type {
            FieldType::String => {
                let value = value.get_str(&self.message).to_string();
                Some(FieldValue::String(value))
            }
            _ => None,
        }
    }
}

pub struct DynamicMessage {
    message: DynamicMessageProto,
    descriptor: Arc<ReflectMessageDescriptor>,
}

impl DynamicMessage {}

impl ReflectMessage for DynamicMessage {
    type Message = DynamicMessageProto;

    fn descriptor(&self) -> &ReflectMessageDescriptor {
        self.descriptor.as_ref()
    }

    fn inner(&self) -> &DynamicMessageProto {
        &self.message
    }

    fn field_value(&self, field: &FieldDescriptor) -> Option<FieldValue> {
        let value = self.message.unknown_fields.get(field.id)?;

        match field.field_type {
            FieldType::String => {
                let value = ProtobufTypeString::get_from_unknown(value)?;
                Some(FieldValue::String(value))
            }
            _ => None,
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

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Message type is not in registry")]
    NotInRegistry,
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

    #[test]
    fn reflect_message_from_any() -> Result<(), failure::Error> {
        let registry = Registry::new();
        registry.register_file_descriptor(
            crate::protos::generated::dynamic::file_descriptor_proto().clone(),
        );

        let msg = TestDynamicMessage {
            string_field: "field1".to_string(),
            ..Default::default()
        };
        let msg_any = message_to_any(&msg)?;

        let dyn_msg = from_any(&registry, &msg_any)?;
        let gen_msg = from_generated(&registry, msg);

        assert_eq!("exocore.test.TestDynamicMessage", dyn_msg.full_name());
        assert_eq!("exocore.test.TestDynamicMessage", gen_msg.full_name());

        assert_eq!(dyn_msg.fields().len(), 3);
        assert_eq!(gen_msg.fields().len(), 3);

        let field1dyn = &dyn_msg.fields()[0];
        assert!(field1dyn.indexed_flag);
        assert_eq!(
            dyn_msg.field_value(field1dyn),
            Some(FieldValue::String("field1".to_string()))
        );
        assert_eq!(
            gen_msg.field_value(field1dyn),
            Some(FieldValue::String("field1".to_string()))
        );

        let field1gen = &gen_msg.fields()[0];
        assert!(field1dyn.indexed_flag);
        assert_eq!(
            dyn_msg.field_value(field1gen),
            Some(FieldValue::String("field1".to_string()))
        );
        assert_eq!(
            gen_msg.field_value(field1gen),
            Some(FieldValue::String("field1".to_string()))
        );

        let field2 = &dyn_msg.fields()[2];
        assert!(!field2.indexed_flag);

        Ok(())
    }
}
