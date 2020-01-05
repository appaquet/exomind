use protobuf;
use protobuf::descriptor::FileDescriptorProto;
use protobuf::types::ProtobufType;
use protobuf::Message;
use std::sync::Arc;

use crate::protos::generated::dynamic::{
    file_descriptor_proto_data, DynamicMessage as DynamicMessageProto, TestDynamicMessage,
    TestStruct,
};

#[test]
fn bleh() {
    let mut msg = TestDynamicMessage::new();
    msg.string_field = "hello world".to_string();
    println!("init msg {:?}", msg);

    msg.set_struct_field(TestStruct {
        field: "yep".to_string(),
        ..Default::default()
    });

    println!("Full name: {}", msg.descriptor().full_name());
    let field = msg.descriptor().field_by_name("string_field");
    let field_type = field.proto().get_field_type();
    println!("Field type {:?}", field_type);
    let out_bytes: Vec<u8> = msg.write_to_bytes().unwrap();

    ///
    let dyn_msg = protobuf::parse_from_bytes::<DynamicMessageProto>(&out_bytes).unwrap();
    println!(
        "deserialized msg {}",
        protobuf::text_format::print_to_string(&msg)
    );

    let file_desc = Arc::new(
        protobuf::parse_from_bytes::<protobuf::descriptor::FileDescriptorProto>(
            file_descriptor_proto_data,
        )
        .unwrap(),
    );
    let msgs_desc = file_desc.get_message_type();

    let msg_desc = &msgs_desc[1];
    println!("Name: {}.{}", file_desc.get_package(), msg_desc.get_name());

    for field in msg_desc.get_field() {
        let indexed = if let Some(indexed_value) = field.get_options().unknown_fields.get(1000) {
            protobuf::types::ProtobufTypeBool::get_from_unknown(indexed_value).unwrap_or(false)
        } else {
            false
        };

        println!(
            "Field={} Type={:?} TypeName={:?} Indexed={:?}",
            field.get_name(),
            field.get_field_type(),
            field.get_type_name(),
            indexed
        );
    }

    let str_value = dyn_msg.unknown_fields.get(1).unwrap();
    let str = protobuf::types::ProtobufTypeString::get_from_unknown(str_value);
    println!("Str: {:?}", str);

    let struct_value = dyn_msg.unknown_fields.get(3).unwrap();

    println!("YAHH");
}

struct Registry {}

struct DynamicMessage {
    message: DynamicMessageProto,
    file_descriptor: Arc<FileDescriptorProto>,
    descriptor_index: usize,
}
