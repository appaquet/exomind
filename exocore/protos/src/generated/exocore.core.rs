#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthToken {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthTokenData {
    #[prost(bytes = "vec", tag = "1")]
    pub cell_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub node_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub signature_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub expiration_date: ::core::option::Option<::prost_types::Timestamp>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LocalNodeConfig {
    #[prost(string, tag = "1")]
    pub keypair: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    #[serde(default)]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "10")]
    #[serde(default)]
    pub listen_addresses: ::core::option::Option<NodeAddresses>,
    #[prost(message, optional, tag = "6")]
    pub addresses: ::core::option::Option<NodeAddresses>,
    #[prost(message, repeated, tag = "7")]
    pub cells: ::prost::alloc::vec::Vec<NodeCellConfig>,
    #[prost(message, optional, tag = "8")]
    #[serde(default)]
    pub store: ::core::option::Option<NodeStoreConfig>,
    #[prost(message, optional, tag = "9")]
    pub chain: ::core::option::Option<ChainConfig>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeAddresses {
    #[prost(string, repeated, tag = "1")]
    #[serde(default)]
    pub p2p: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "2")]
    #[serde(default)]
    pub http: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeCellConfig {
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub id: ::prost::alloc::string::String,
    #[prost(oneof = "node_cell_config::Location", tags = "1")]
    #[serde(flatten)]
    pub location: ::core::option::Option<node_cell_config::Location>,
}
/// Nested message and enum types in `NodeCellConfig`.
pub mod node_cell_config {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Location {
        #[prost(message, tag = "1")]
        Inline(super::CellConfig),
    }
}
/// Entity store configuration for the node (i.e. not global)
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeStoreConfig {
    /// Entity index config.
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub index: ::core::option::Option<EntityIndexConfig>,
    /// Maximum number of queries to execute in parallel.
    #[prost(message, optional, tag = "2")]
    #[serde(default)]
    pub query_parallelism: ::core::option::Option<u32>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainConfig {
    /// Maximum size in bytes per segment. This is a soft limit since the last
    /// block could overflow that maximum. This should be small enough so
    /// that a few segments can fit in allocable virtual space on 32b
    /// systems. See `segment_max_open` for maximum concurrently opened
    /// segments.
    #[prost(message, optional, tag = "1")]
    pub segment_max_size: ::core::option::Option<u64>,
    /// Maximum number of segments concurrently mmap. On 64b systems, where
    /// virtual memory isn't a problem, this can be high. But on 32b
    /// systems, one should aim to have maximum ~1-2gb of concurrently mmap
    /// segments. See `segment_max_size` for maximum size per segment.
    #[prost(message, optional, tag = "2")]
    pub segment_max_open_mmap: ::core::option::Option<u32>,
}
/// Configuration of the entity index
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityIndexConfig {
    /// What is the minimum depth that a block needs to be the chain to be
    /// indexed. This is required to lower the odds that we are going to
    /// revert the block if our local chain forked.
    ///
    /// `CommitManagerConfig`.`operations_cleanup_after_block_depth`
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub chain_index_min_depth: ::core::option::Option<u64>,
    /// If specified, prevent indexing every new block on each commit.
    /// Operations will be kept in pending index for a bit longer and
    /// preventing the costly chain index modification.
    #[prost(message, optional, tag = "2")]
    #[serde(default)]
    pub chain_index_depth_leeway: ::core::option::Option<u64>,
    /// Specifies the interval at which new blocks in the chain get indexed.
    /// New blocks may not necessarily get immediately indexed if they don't
    /// fall in the interval of `chain_index_min_depth` and
    /// `chain_index_depth_leeway`.
    ///
    /// Indexation can also be prevented if user queries were recently executed
    /// (see `chain_index_deferred_query_secs`)
    ///
    /// If '0' is specified, deferred indexation is disabled and blocks are
    /// indexed when the chain layer emits events.
    #[prost(message, optional, tag = "6")]
    #[serde(default)]
    pub chain_index_deferred_interval_secs: ::core::option::Option<u64>,
    /// Specifies the minimum interval to wait before indexing chain blocks
    /// after receiving a user query. It prevents potential slow downs caused
    /// by chain indexation if a user query get executed frequently.
    #[prost(message, optional, tag = "7")]
    #[serde(default)]
    pub chain_index_deferred_query_secs: ::core::option::Option<u64>,
    /// Specifies the maximum interval for which indexation may be blocked by
    /// incoming user queries.
    #[prost(message, optional, tag = "8")]
    #[serde(default)]
    pub chain_index_deferred_max_secs: ::core::option::Option<u64>,
    /// Configuration for the in-memory traits index that are in the pending
    /// store
    #[prost(message, optional, tag = "3")]
    #[serde(default)]
    pub pending_index: ::core::option::Option<MutationIndexConfig>,
    /// Configuration for the persisted traits index that are in the chain
    #[prost(message, optional, tag = "4")]
    #[serde(default)]
    pub chain_index: ::core::option::Option<MutationIndexConfig>,
    /// Configuration for the entity garbage collector.
    #[prost(message, optional, tag = "5")]
    #[serde(default)]
    pub garbage_collector: ::core::option::Option<EntityGarbageCollectorConfig>,
}
/// Trait index configuration
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutationIndexConfig {
    /// Number of indexing threads.
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub indexer_num_threads: ::core::option::Option<u32>,
    /// Maximum heap size of each indexing thread.
    #[prost(message, optional, tag = "2")]
    #[serde(default)]
    pub indexer_heap_size_bytes: ::core::option::Option<u32>,
    /// Page size of results iterator.
    #[prost(message, optional, tag = "3")]
    #[serde(default)]
    pub entity_mutations_cache_size: ::core::option::Option<u32>,
}
/// Configuration for entity garbage collector.
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityGarbageCollectorConfig {
    /// How often the garbage collection process will run in seconds.
    ///
    /// Since garbage collection doesn't happen on the whole index, but only on
    /// entities that got flagged during search, it is better to run more
    /// often than less. `GarbageCollectorConfig::queue_size` can be tweaked
    /// to control rate of collection.
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub run_interval_secs: ::core::option::Option<u32>,
    /// Size of the queue of entities to be collected.
    #[prost(message, optional, tag = "2")]
    #[serde(default)]
    pub queue_size: ::core::option::Option<u32>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CellConfig {
    #[prost(string, tag = "1")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub keypair: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    #[serde(default)]
    pub id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "6")]
    pub nodes: ::prost::alloc::vec::Vec<CellNodeConfig>,
    #[prost(message, repeated, tag = "7")]
    #[serde(default)]
    pub apps: ::prost::alloc::vec::Vec<CellApplicationConfig>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CellNodeConfig {
    #[prost(message, optional, tag = "1")]
    pub node: ::core::option::Option<NodeConfig>,
    #[prost(enumeration = "cell_node_config::Role", repeated, tag = "2")]
    #[serde(default)]
    pub roles: ::prost::alloc::vec::Vec<i32>,
}
/// Nested message and enum types in `CellNodeConfig`.
pub mod cell_node_config {
    #[derive(
        Serialize,
        Deserialize,
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration,
    )]
    #[repr(i32)]
    pub enum Role {
        InvalidRole = 0,
        ChainRole = 1,
        StoreRole = 2,
        AppHostRole = 3,
    }
    impl Role {
        /// String value of the enum field names used in the ProtoBuf
        /// definition.
        ///
        /// The values are not transformed in any way and thus are considered
        /// stable (if the ProtoBuf definition does not change) and safe
        /// for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Role::InvalidRole => "INVALID_ROLE",
                Role::ChainRole => "CHAIN_ROLE",
                Role::StoreRole => "STORE_ROLE",
                Role::AppHostRole => "APP_HOST_ROLE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "INVALID_ROLE" => Some(Self::InvalidRole),
                "CHAIN_ROLE" => Some(Self::ChainRole),
                "STORE_ROLE" => Some(Self::StoreRole),
                "APP_HOST_ROLE" => Some(Self::AppHostRole),
                _ => None,
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeConfig {
    #[prost(string, tag = "1")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub addresses: ::core::option::Option<NodeAddresses>,
}
#[derive(Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CellApplicationConfig {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub package_url: ::prost::alloc::string::String,
    #[prost(oneof = "cell_application_config::Location", tags = "5")]
    #[serde(flatten)]
    pub location: ::core::option::Option<cell_application_config::Location>,
}
/// Nested message and enum types in `CellApplicationConfig`.
pub mod cell_application_config {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Location {
        /// Manifest is inline within the config.
        #[prost(message, tag = "5")]
        Inline(super::super::apps::Manifest),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub build_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(bool, tag = "3")]
    pub debug: bool,
    #[prost(string, tag = "4")]
    pub rust_version: ::prost::alloc::string::String,
}
