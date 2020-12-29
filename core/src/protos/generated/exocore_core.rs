#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthToken {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
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
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
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
    #[prost(string, tag = "5")]
    #[serde(default)]
    pub path: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "6")]
    pub addresses: ::core::option::Option<NodeAddresses>,
    #[prost(message, repeated, tag = "7")]
    pub cells: ::prost::alloc::vec::Vec<NodeCellConfig>,
    #[prost(message, optional, tag = "8")]
    #[serde(default)]
    pub store: ::core::option::Option<NodeStoreConfig>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct NodeAddresses {
    #[prost(string, repeated, tag = "1")]
    #[serde(default)]
    pub p2p: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "2")]
    #[serde(default)]
    pub http: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct NodeCellConfig {
    #[prost(oneof = "node_cell_config::Location", tags = "1, 2")]
    #[serde(flatten)]
    pub location: ::core::option::Option<node_cell_config::Location>,
}
/// Nested message and enum types in `NodeCellConfig`.
pub mod node_cell_config {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Location {
        #[prost(message, tag = "1")]
        Inline(super::CellConfig),
        #[prost(string, tag = "2")]
        Path(::prost::alloc::string::String),
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct NodeStoreConfig {
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub index: ::core::option::Option<EntityIndexConfig>,
}
//// Configuration of the entities index
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct EntityIndexConfig {
    //// What is the minimum depth that a block needs to be the chain to be
    //// indexed. This is required to lower the odds that we are going to
    //// revert the block if our local chain forked.
    ////
    //// `CommitManagerConfig`.`operations_cleanup_after_block_depth`
    #[prost(uint64, tag = "1")]
    pub chain_index_min_depth: u64,
    //// If specified, prevent indexing every new block on each commit.
    //// Operations will be kept in pending index for a bit longer and
    //// preventing the costly chain index modification.
    #[prost(uint64, tag = "2")]
    pub chain_index_depth_leeway: u64,
    //// Configuration for the in-memory traits index that are in the pending
    //// store
    #[prost(message, optional, tag = "3")]
    pub pending_index: ::core::option::Option<MutationIndexConfig>,
    //// Configuration for the persisted traits index that are in the chain
    #[prost(message, optional, tag = "4")]
    pub chain_index: ::core::option::Option<MutationIndexConfig>,
}
//// Trait index configuration
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct MutationIndexConfig {
    #[prost(uint32, tag = "1")]
    #[serde(default)]
    pub indexer_num_threads: u32,
    #[prost(uint32, tag = "2")]
    #[serde(default)]
    pub indexer_heap_size_bytes: u32,
    #[prost(uint32, tag = "3")]
    #[serde(default)]
    pub iterator_page_size: u32,
    #[prost(uint32, tag = "4")]
    #[serde(default)]
    pub iterator_max_pages: u32,
    #[prost(uint32, tag = "5")]
    #[serde(default)]
    pub entity_mutations_cache_size: u32,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
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
    #[prost(string, tag = "5")]
    #[serde(default)]
    pub path: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "6")]
    pub nodes: ::prost::alloc::vec::Vec<CellNodeConfig>,
    #[prost(message, repeated, tag = "7")]
    #[serde(default)]
    pub apps: ::prost::alloc::vec::Vec<CellApplicationConfig>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
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
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct CellApplicationConfig {
    #[prost(oneof = "cell_application_config::Location", tags = "1, 2")]
    #[serde(flatten)]
    pub location: ::core::option::Option<cell_application_config::Location>,
}
/// Nested message and enum types in `CellApplicationConfig`.
pub mod cell_application_config {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Location {
        #[prost(message, tag = "1")]
        Inline(super::super::apps::Manifest),
        #[prost(string, tag = "2")]
        Path(::prost::alloc::string::String),
    }
}
