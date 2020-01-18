use protoc_rust::Customize;
use std::env;

fn main() {
    if env::var("GENERATE_PROTOS").is_ok() {
        let capn_protos_file = vec![
            "protos/common.capnp",
            "protos/data_chain.capnp",
            "protos/data_transport.capnp",
            "protos/index_transport.capnp",
        ];
        for proto_file in capn_protos_file {
            capnpc::CompilerCommand::new()
                .file(proto_file)
                .run()
                .expect(&format!("compiling {} schema", proto_file));
        }

        let protobuf_protos_file = vec![
            "protos/reflect.proto",
            "protos/exocore/index/entity.proto",
            "protos/exocore/index/query.proto",
            "protos/exocore/index/results.proto",
        ];
        protoc_rust::run(protoc_rust::Args {
            out_dir: "src/protos/generated",
            input: &protobuf_protos_file,
            includes: &["protos"],
            customize: Customize {
                expose_fields: Some(true),
                ..Default::default()
            },
        })
        .expect("protoc error");
    }
}
