use super::reflect::{FieldDescriptor, FieldType, ReflectMessageDescriptor};
use super::Error;
use protobuf;
use protobuf::descriptor::{
    DescriptorProto, FieldDescriptorProto_Type, FileDescriptorProto, FileDescriptorSet,
};
use protobuf::types::{ProtobufType, ProtobufTypeBool};
use protobuf::Message;
use std::collections::HashMap;
use std::sync::Arc;
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

    pub fn new_with_exocore_types() -> Registry {
        let reg = Registry::new();

        reg.register_file_descriptor_set_bytes(super::generated::INDEX_FDSET)
            .expect("Couldn't register exocore_index FileDescriptorProto");

        reg.register_file_descriptor_set_bytes(super::generated::TEST_FDSET)
            .expect("Couldn't register exocore_test FileDescriptorProto");

        reg
    }

    pub fn register_file_descriptor_set(&self, fd_set: &FileDescriptorSet) {
        for fd in fd_set.get_file() {
            self.register_file_descriptor(fd.clone());
        }
    }

    pub fn register_file_descriptor_set_bytes<R: std::io::Read>(
        &self,
        fd_set_bytes: R,
    ) -> Result<(), Error> {
        let mut bytes = fd_set_bytes;
        let fd_set = protobuf::parse_from_reader(&mut bytes)
            .map_err(|err| Error::StepanProtobuf(Arc::new(err)))?;

        self.register_file_descriptor_set(&fd_set);

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_exocore_types() {
        let reg = Registry::new_with_exocore_types();
        let entity = reg.get_message_descriptor("exocore.index.Entity").unwrap();
        assert_eq!(entity.name, "exocore.index.Entity");
        assert!(!entity.fields.is_empty());
    }
}
