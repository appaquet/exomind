use std::env;

fn main() {
    if env::var("GENERATE_PROTOS").is_ok() {
        {
            let capn_protos_file = vec![
                "../protos/common.capnp",
                "../protos/data_chain.capnp",
                "../protos/data_transport.capnp",
                "../protos/store_transport.capnp",
            ];
            for proto_file in capn_protos_file {
                capnpc::CompilerCommand::new()
                    .file(proto_file)
                    .run()
                    .unwrap_or_else(|_| panic!("compiling {} schema", proto_file));
            }
        }

        {
            let prost_protos_file = vec![
                "../protos/exocore/store/entity.proto",
                "../protos/exocore/store/query.proto",
                "../protos/exocore/store/mutation.proto",
                "../protos/exocore/test/test.proto",
                "../protos/exocore/core/auth.proto",
                "../protos/exocore/core/config.proto",
                "../protos/exocore/apps/manifest.proto",
            ];

            let mut config = prost_build::Config::new();

            // add serde serializability on some types
            config
                .type_attribute("LocalNodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeAddresses", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeCellConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute(
                    "NodeCellConfig.location",
                    "#[derive(Serialize, Deserialize)]",
                )
                .type_attribute("CellConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellNodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellNodeConfig.Role", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellApplicationConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute(
                    "CellApplicationConfig.location",
                    "#[derive(Serialize, Deserialize)]",
                )
                .type_attribute("Manifest", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ManifestSchema", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ManifestSchema.source", "#[derive(Serialize, Deserialize)]");

            // default fields
            config
                .field_attribute("LocalNodeConfig.name", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.id", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.path", "#[serde(default)]")
                .field_attribute("NodeConfig.name", "#[serde(default)]")
                .field_attribute("NodeConfig.id", "#[serde(default)]")
                .field_attribute("NodeAddresses.p2p", "#[serde(default)]")
                .field_attribute("NodeAddresses.http", "#[serde(default)]")
                .field_attribute("CellConfig.name", "#[serde(default)]")
                .field_attribute("CellConfig.keypair", "#[serde(default)]")
                .field_attribute("CellConfig.id", "#[serde(default)]")
                .field_attribute("CellConfig.path", "#[serde(default)]")
                .field_attribute("CellConfig.apps", "#[serde(default)]")
                .field_attribute("CellNodeConfig.roles", "#[serde(default)]")
                .field_attribute("Manifest.path", "#[serde(default)]")
                .field_attribute("Manifest.schemas", "#[serde(default)]");

            config
                .compile_protos(&prost_protos_file, &["../protos/"])
                .expect("prost error");
        }
    }
}
