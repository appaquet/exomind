extern crate capnpc;
use std::env;

fn main() {
    if env::var("GENERATE_PROTOS").is_ok() {
        let protos_file = vec![
            "protos/common.capnp",
            "protos/data_chain.capnp",
            "protos/data_transport.capnp",
            "protos/index_transport.capnp",
        ];

        for proto_file in protos_file {
            ::capnpc::CompilerCommand::new()
                .file(proto_file)
                .run()
                .expect(&format!("compiling {} schema", proto_file));
        }
    }
}
