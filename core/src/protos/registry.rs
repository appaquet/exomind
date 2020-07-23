use super::reflect::{FieldDescriptor, FieldType, ReflectMessageDescriptor};
use super::Error;
use protobuf::descriptor::{
    DescriptorProto, FieldDescriptorProto, FieldDescriptorProto_Type, FileDescriptorProto,
    FileDescriptorSet,
};
use protobuf::types::{ProtobufType, ProtobufTypeBool};
use protobuf::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

type MessageDescriptorsMap = HashMap<String, Arc<ReflectMessageDescriptor>>;

pub struct Registry {
    message_descriptors: RwLock<MessageDescriptorsMap>,
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
        let mut fields = HashMap::new();
        for field in msg_descriptor.get_field() {
            let field_type = match field.get_field_type() {
                FieldDescriptorProto_Type::TYPE_STRING => FieldType::String,
                FieldDescriptorProto_Type::TYPE_INT32 => FieldType::Int32,
                FieldDescriptorProto_Type::TYPE_UINT32 => FieldType::Uint32,
                FieldDescriptorProto_Type::TYPE_INT64 => FieldType::Int64,
                FieldDescriptorProto_Type::TYPE_UINT64 => FieldType::Uint64,
                FieldDescriptorProto_Type::TYPE_MESSAGE => {
                    let typ = field.get_type_name().trim_start_matches('.');
                    match typ {
                        "google.protobuf.Timestamp" => FieldType::DateTime,
                        "exocore.index.Reference" => FieldType::Reference,
                        _ => FieldType::Message(typ.to_string()),
                    }
                }
                _ => continue,
            };

            let id = field.get_number() as u32;
            fields.insert(
                id,
                FieldDescriptor {
                    id,
                    name: field.get_name().to_string(),
                    field_type,

                    // see exocore/index/options.proto
                    indexed_flag: Registry::field_has_option(field, 1373),
                    sorted_flag: Registry::field_has_option(field, 1374),
                    text_flag: Registry::field_has_option(field, 1375),
                    groups: Registry::get_field_u32s_option(field, 1376),
                },
            );
        }

        let descriptor = Arc::new(ReflectMessageDescriptor {
            name: full_name.clone(),
            fields,
            message: msg_descriptor,
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
            .ok_or_else(|| Error::NotInRegistry(full_name.to_string()))
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

    pub fn message_descriptors(&self) -> Vec<Arc<ReflectMessageDescriptor>> {
        let message_descriptors = self.message_descriptors.read().unwrap();
        message_descriptors.values().cloned().collect()
    }

    fn field_has_option(field: &FieldDescriptorProto, option_field_id: u32) -> bool {
        if let Some(unknown_value) = field.get_options().unknown_fields.get(option_field_id) {
            ProtobufTypeBool::get_from_unknown(unknown_value).unwrap_or(false)
        } else {
            false
        }
    }

    fn get_field_u32s_option(field: &FieldDescriptorProto, option_field_id: u32) -> Vec<u32> {
        if let Some(unknown_value) = field.get_options().unknown_fields.get(option_field_id) {
            unknown_value.varint.iter().map(|&v| v as u32).collect()
        } else {
            vec![]
        }
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

        let desc = reg.message_descriptors();
        assert!(desc.len() > 20);
    }

    #[test]
    fn field_options() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let descriptor = registry.get_message_descriptor("exocore.test.TestMessage")?;

        // see `protos/exocore/test/test.proto`
        assert_eq!(descriptor.fields.get(&1).unwrap().text_flag, true);
        assert_eq!(descriptor.fields.get(&2).unwrap().text_flag, false);

        assert_eq!(descriptor.fields.get(&8).unwrap().indexed_flag, true);
        assert_eq!(descriptor.fields.get(&9).unwrap().indexed_flag, false);

        assert_eq!(descriptor.fields.get(&18).unwrap().sorted_flag, true);
        assert_eq!(descriptor.fields.get(&11).unwrap().sorted_flag, false);

        assert!(descriptor.fields.get(&19).unwrap().groups.is_empty());
        assert_eq!(descriptor.fields.get(&20).unwrap().groups, vec![1]);
        assert_eq!(descriptor.fields.get(&21).unwrap().groups, vec![1, 2]);

        Ok(())
    }
}
