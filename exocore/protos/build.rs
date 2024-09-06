fn main() {
    if std::env::var("GENERATE_PROTOS").is_ok() {
        {
            let capn_protos_file = vec![
                "./capn/common.capnp",
                "./capn/data_chain.capnp",
                "./capn/data_transport.capnp",
                "./capn/store_transport.capnp",
            ];
            for proto_file in capn_protos_file {
                capnpc::CompilerCommand::new()
                    .file(proto_file)
                    .output_path("./src/generated/")
                    .run()
                    .unwrap_or_else(|err| panic!("compiling {} schema: {}", proto_file, err));
            }
        }

        {
            let prost_protos_file = vec![
                "./protobuf/exocore/store/entity.proto",
                "./protobuf/exocore/store/query.proto",
                "./protobuf/exocore/store/mutation.proto",
                "./protobuf/exocore/test/test.proto",
                "./protobuf/exocore/core/auth.proto",
                "./protobuf/exocore/core/config.proto",
                "./protobuf/exocore/core/build.proto",
                "./protobuf/exocore/apps/manifest.proto",
                "./protobuf/exocore/apps/runtime.proto",
            ];

            let mut config = prost_build::Config::new();

            // add serde annotations on some types and fields
            config
                .type_attribute("LocalNodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeAddresses", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeCellConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute(
                    "NodeCellConfig.location",
                    "#[derive(Serialize, Deserialize)]",
                )
                .type_attribute(
                    "NodeCellConfig.location",
                    "#[serde(rename_all = \"lowercase\")]",
                )
                .field_attribute("NodeCellConfig.location", "#[serde(flatten)]")
                .field_attribute("NodeCellConfig.id", "#[serde(default)]") // TODO: Remove once migrated to new cell config
                .type_attribute("NodeStoreConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ChainConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("EntityIndexConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("MutationIndexConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("EntityGarbageCollectorConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellNodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellNodeConfig.Role", "#[derive(Serialize, Deserialize)]")
                .type_attribute("NodeConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute("CellApplicationConfig", "#[derive(Serialize, Deserialize)]")
                .type_attribute(
                    "CellApplicationConfig.location",
                    "#[derive(Serialize, Deserialize)]",
                )
                .type_attribute(
                    "CellApplicationConfig.location",
                    "#[serde(rename_all = \"lowercase\")]",
                )
                .field_attribute("CellApplicationConfig.location", "#[serde(flatten)]")
                .type_attribute("Manifest", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ManifestModule", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ManifestSchema", "#[derive(Serialize, Deserialize)]")
                .type_attribute("ManifestSchema.source", "#[derive(Serialize, Deserialize)]")
                .type_attribute(
                    "ManifestSchema.source",
                    "#[serde(rename_all = \"lowercase\")]",
                )
                .field_attribute("ManifestSchema.source", "#[serde(flatten)]")
                .field_attribute("ManifestSchema.source.bytes", "#[serde(serialize_with = \"crate::base64::as_base64\", deserialize_with = \"crate::base64::from_base64\")]")
                .field_attribute("LocalNodeConfig.name", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.id", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.listen_addresses", "#[serde(default)]")
                .field_attribute("LocalNodeConfig.store", "#[serde(default)]")
                .field_attribute("NodeStoreConfig.index", "#[serde(default)]")
                .field_attribute("NodeStoreConfig.query_parallelism", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index_min_depth", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index_depth_leeway", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index_deferred_interval_secs", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index_deferred_query_secs", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index_deferred_max_secs", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.pending_index", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.chain_index", "#[serde(default)]")
                .field_attribute("EntityIndexConfig.garbage_collector", "#[serde(default)]")
                .field_attribute("MutationIndexConfig.indexer_num_threads", "#[serde(default)]")
                .field_attribute("MutationIndexConfig.indexer_heap_size_bytes", "#[serde(default)]")
                .field_attribute("MutationIndexConfig.entity_mutations_cache_size", "#[serde(default)]")
                .field_attribute("EntityGarbageCollectorConfig.run_interval_secs", "#[serde(default)]")
                .field_attribute("EntityGarbageCollectorConfig.queue_size", "#[serde(default)]")
                .field_attribute("NodeConfig.name", "#[serde(default)]")
                .field_attribute("NodeConfig.id", "#[serde(default)]")
                .field_attribute("NodeAddresses.p2p", "#[serde(default)]")
                .field_attribute("NodeAddresses.http", "#[serde(default)]")
                .field_attribute("CellConfig.name", "#[serde(default)]")
                .field_attribute("CellConfig.keypair", "#[serde(default)]")
                .field_attribute("CellConfig.id", "#[serde(default)]")
                .field_attribute("CellConfig.apps", "#[serde(default)]")
                .field_attribute("CellNodeConfig.roles", "#[serde(default)]")
                .field_attribute("Manifest.schemas", "#[serde(default)]")
                .field_attribute("ManifestModule.multihash", "#[serde(default)]");

            config
                .out_dir("./src/generated/")
                .compile_protos(&prost_protos_file, &["./protobuf/"])
                .expect("prost error");
        }
    }
}
