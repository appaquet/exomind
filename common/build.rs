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

        let prost_protos_file = vec![
            "protos/exocore/index/entity.proto",
            "protos/exocore/index/query.proto",
            "protos/exocore/index/mutation.proto",
            "protos/exocore/test/test.proto",
        ];
        prost_build::compile_protos(&prost_protos_file, &["protos/"]).expect("prost error");
    }
}
