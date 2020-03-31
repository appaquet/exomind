use std::env;

fn main() {
    if env::var("GENERATE_PROTOS").is_ok() {
        {
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
                    .unwrap_or_else(|_| panic!("compiling {} schema", proto_file));
            }
        }

        {
            // protos without serde compatibility
            let prost_protos_file = vec![
                "protos/exocore/index/entity.proto",
                "protos/exocore/index/query.proto",
                "protos/exocore/index/mutation.proto",
                "protos/exocore/test/test.proto",
            ];
            prost_build::compile_protos(&prost_protos_file, &["protos/"]).expect("prost error");
        }

        {
            // protos with serde compatibility
            let prost_protos_file = vec![
                "protos/exocore/core/config.proto",
                "protos/exocore/apps/manifest.proto",
            ];
            let mut config = prost_build::Config::new();
            config.type_attribute(".", "#[derive(Serialize, Deserialize)]");

            // default fields
            config
                .field_attribute("LocalNodeConfig.name", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.path", "#[serde(default)]")
                .field_attribute("NodeConfig.name", "#[serde(default)]")
                .field_attribute("NodeConfig.addresses", "#[serde(default)]")
                .field_attribute("CellConfig.name", "#[serde(default)]")
                .field_attribute("CellConfig.keypair", "#[serde(default)]")
                .field_attribute("CellConfig.path", "#[serde(default)]")
                .field_attribute("CellConfig.apps", "#[serde(default)]")
                .field_attribute("CellNodeConfig.roles", "#[serde(default)]")
                .field_attribute("Manifest.schemas", "#[serde(default)]")
                .field_attribute("Manifest.path", "#[serde(default)]");

            config
                .compile_protos(&prost_protos_file, &["protos/"])
                .expect("prost error");
        }
    }
}
