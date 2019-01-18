extern crate capnpc;

fn main() {
    let protos_file = vec![
        "proto/common.capnp",
        "proto/data_chain.capnp",
        "proto/data_transport.capnp",
    ];

    for proto_file in protos_file {
        ::capnpc::CompilerCommand::new()
            .file(proto_file)
            .edition(capnpc::RustEdition::Rust2018)
            .run()
            .expect(&format!("compiling {} schema", proto_file));
    }
}
