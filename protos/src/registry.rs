use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use protobuf::{
    descriptor::{FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet},
    reflect::FileDescriptor,
    Message, MessageFull, UnknownValueRef,
};

use super::{
    reflect::{FieldDescriptor, FieldType, ReflectMessageDescriptor},
    Error,
};

type MessageDescriptorsMap = HashMap<String, Arc<ReflectMessageDescriptor>>;
type FileDescriptorsMap = HashMap<String, FileDescriptor>;

pub struct Registry {
    message_descriptors: RwLock<MessageDescriptorsMap>,
    file_descriptors: RwLock<FileDescriptorsMap>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            message_descriptors: RwLock::new(HashMap::new()),
            file_descriptors: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_with_exocore_types() -> Registry {
        let reg = Registry::new();

        reg.register_well_knowns();

        reg.register_file_descriptor_set_bytes(super::generated::STORE_FDSET)
            .expect("Couldn't register exocore_store FileDescriptorProto");

        reg.register_file_descriptor_set_bytes(super::generated::TEST_FDSET)
            .expect("Couldn't register exocore_test FileDescriptorProto");

        reg
    }

    pub fn register_well_knowns(&self) {
        let fds = &[
            protobuf::well_known_types::timestamp::Timestamp::descriptor(),
            protobuf::well_known_types::any::Any::descriptor(),
            FileDescriptorProto::descriptor(),
        ];

        for fd in fds {
            self.register_file_descriptor(fd.file_descriptor_proto().clone());
        }
    }

    pub fn register_file_descriptor_set(&self, fd_set: &FileDescriptorSet) {
        let fds = protobuf::reflect::FileDescriptor::new_dynamic_fds(
            fd_set.file.clone(),
            self.dependencies().as_ref(),
        )
        .expect("FIX ME");

        for fd in &fds {
            {
                let mut file_descriptors = self.file_descriptors.write().unwrap();
                file_descriptors.insert(fd.name().to_string(), fd.clone());
            }

            for msg_descriptor in fd.messages() {
                let full_name = format!("{}.{}", fd.package(), msg_descriptor.name(),);
                self.register_message_descriptor(full_name, msg_descriptor);
            }
        }
    }

    fn dependencies(&self) -> Vec<FileDescriptor> {
        let fds = self.file_descriptors.read().unwrap();
        fds.values().cloned().collect()
    }

    pub fn register_file_descriptor_set_bytes<R: std::io::Read>(
        &self,
        fd_set_bytes: R,
    ) -> Result<(), Error> {
        let mut bytes = fd_set_bytes;
        let fd_set = FileDescriptorSet::parse_from_reader(&mut bytes)?;

        self.register_file_descriptor_set(&fd_set);

        Ok(())
    }

    pub fn register_file_descriptor(&self, file_descriptor_proto: FileDescriptorProto) {
        let fd = protobuf::reflect::FileDescriptor::new_dynamic(
            file_descriptor_proto,
            self.dependencies().as_ref(),
        )
        .expect("FIX ME");

        {
            let mut file_descriptors = self.file_descriptors.write().unwrap();
            file_descriptors.insert(fd.name().to_string(), fd.clone());
        }

        for msg_descriptor in fd.messages() {
            let full_name = format!("{}.{}", fd.package(), msg_descriptor.name(),);
            self.register_message_descriptor(full_name, msg_descriptor);
        }
    }

    pub fn register_message_descriptor(
        &self,
        full_name: String,
        msg_descriptor: protobuf::reflect::MessageDescriptor,
    ) -> Arc<ReflectMessageDescriptor> {
        for sub_msg in msg_descriptor.nested_messages() {
            let sub_full_name = format!("{}.{}", full_name, sub_msg.name());
            self.register_message_descriptor(sub_full_name, sub_msg.clone());
        }

        let mut fields = HashMap::new();
        for field in msg_descriptor.fields() {
            let field_proto = field.proto();

            use protobuf::descriptor::field_descriptor_proto::Type as ProtoFieldType;
            let mut field_type = match field_proto.type_.map(|e| e.enum_value()) {
                Some(Ok(ProtoFieldType::TYPE_STRING)) => FieldType::String,
                Some(Ok(ProtoFieldType::TYPE_INT32)) => FieldType::Int32,
                Some(Ok(ProtoFieldType::TYPE_UINT32)) => FieldType::Uint32,
                Some(Ok(ProtoFieldType::TYPE_INT64)) => FieldType::Int64,
                Some(Ok(ProtoFieldType::TYPE_UINT64)) => FieldType::Uint64,
                Some(Ok(ProtoFieldType::TYPE_MESSAGE)) => {
                    let typ = field_proto.type_name().trim_start_matches('.');
                    match typ {
                        "google.protobuf.Timestamp" => FieldType::DateTime,
                        "exocore.store.Reference" => FieldType::Reference,
                        _ => FieldType::Message(typ.to_string()),
                    }
                }

                _ => continue,
            };

            if field_proto.label()
                == protobuf::descriptor::field_descriptor_proto::Label::LABEL_REPEATED
            {
                field_type = FieldType::Repeated(Box::new(field_type));
            }

            if let Some(number) = field_proto.number {
                let id = number as u32;
                fields.insert(
                    id,
                    FieldDescriptor {
                        id,
                        descriptor: field.clone(),
                        name: field.name().to_string(),
                        field_type,

                        // see exocore/store/options.proto
                        indexed_flag: Registry::field_has_option(field_proto, 1373),
                        sorted_flag: Registry::field_has_option(field_proto, 1374),
                        text_flag: Registry::field_has_option(field_proto, 1375),
                        groups: Registry::get_field_u32s_option(field_proto, 1376),
                    },
                );
            }
        }

        let short_names = Registry::get_message_strings_option(&msg_descriptor, 1377);
        let descriptor = Arc::new(ReflectMessageDescriptor {
            name: full_name.clone(),
            fields,
            message: msg_descriptor,

            // see exocore/store/options.proto
            short_names,
        });

        let mut file_descriptors = self.file_descriptors.write().unwrap();
        let fd = descriptor.message.file_descriptor();
        file_descriptors.insert(fd.name().to_string(), fd.clone());

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

    pub fn message_descriptors(&self) -> Vec<Arc<ReflectMessageDescriptor>> {
        let message_descriptors = self.message_descriptors.read().unwrap();
        message_descriptors.values().cloned().collect()
    }

    fn field_has_option(field: &FieldDescriptorProto, option_field_id: u32) -> bool {
        if let Some(UnknownValueRef::Varint(v)) =
            field.options.unknown_fields().get(option_field_id)
        {
            v == 1
        } else {
            false
        }
    }

    fn get_field_u32s_option(field: &FieldDescriptorProto, option_field_id: u32) -> Vec<u32> {
        let mut ret = Vec::new();
        for (field_id, value) in field.options.unknown_fields().iter() {
            // unfortunately, doesn't allow getting multiple values for one field other than
            // iterating on all options
            if field_id != option_field_id {
                continue;
            }

            match value {
                UnknownValueRef::Varint(v) => {
                    ret.push(v as u32);
                }
                UnknownValueRef::LengthDelimited(values) => {
                    for value in values {
                        ret.push(*value as u32);
                    }
                }
                _ => (),
            }
        }
        ret
    }

    fn get_message_strings_option(
        msg_desc: &protobuf::reflect::MessageDescriptor,
        option_field_id: u32,
    ) -> Vec<String> {
        let mut ret = Vec::new();
        for (field_id, value) in msg_desc.proto().options.unknown_fields().iter() {
            if let UnknownValueRef::LengthDelimited(bytes) = value {
                // unfortunately, doesn't allow getting multiple values for one field other than
                // iterating on all options
                if field_id == option_field_id {
                    ret.push(String::from_utf8_lossy(bytes).to_string());
                }
            }
        }
        ret
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
        let entity = reg.get_message_descriptor("exocore.store.Entity").unwrap();
        assert_eq!(entity.name, "exocore.store.Entity");
        assert!(!entity.fields.is_empty());

        let desc = reg.message_descriptors();
        assert!(desc.len() > 20);
    }

    #[test]
    fn field_and_msg_options() -> anyhow::Result<()> {
        let registry = Registry::new_with_exocore_types();

        let descriptor = registry.get_message_descriptor("exocore.test.TestMessage")?;

        // see `protos/exocore/test/test.proto`
        assert_eq!(descriptor.short_names, vec!["test".to_string()]);

        assert!(descriptor.fields.get(&1).unwrap().text_flag);
        assert!(!descriptor.fields.get(&2).unwrap().text_flag);

        assert!(descriptor.fields.get(&8).unwrap().indexed_flag);
        assert!(!descriptor.fields.get(&9).unwrap().indexed_flag);

        assert!(descriptor.fields.get(&18).unwrap().sorted_flag);
        assert!(!descriptor.fields.get(&11).unwrap().sorted_flag);

        assert!(descriptor.fields.get(&19).unwrap().groups.is_empty());
        assert_eq!(descriptor.fields.get(&20).unwrap().groups, vec![1]);
        assert_eq!(descriptor.fields.get(&21).unwrap().groups, vec![1, 2]);

        Ok(())
    }
}
