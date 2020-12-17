#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthToken {
    #[prost(bytes, tag = "1")]
    pub data: std::vec::Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub signature: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthTokenData {
    #[prost(bytes, tag = "1")]
    pub cell_id: std::vec::Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub node_id: std::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub signature_date: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub expiration_date: ::std::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct LocalNodeConfig {
    #[prost(string, tag = "1")]
    pub keypair: std::string::String,
    #[prost(string, tag = "2")]
    pub public_key: std::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub name: std::string::String,
    #[prost(string, tag = "4")]
    #[serde(default)]
    pub id: std::string::String,
    #[prost(string, tag = "5")]
    #[serde(default)]
    pub path: std::string::String,
    #[prost(message, optional, tag = "6")]
    pub addresses: ::std::option::Option<NodeAddresses>,
    #[prost(message, repeated, tag = "7")]
    pub cells: ::std::vec::Vec<NodeCellConfig>,
    #[prost(message, optional, tag = "8")]
    #[serde(default)]
    pub store: ::std::option::Option<NodeStoreConfig>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeAddresses {
    #[prost(string, repeated, tag = "1")]
    #[serde(default)]
    pub p2p: ::std::vec::Vec<std::string::String>,
    #[prost(string, repeated, tag = "2")]
    #[serde(default)]
    pub http: ::std::vec::Vec<std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeCellConfig {
    #[prost(oneof = "node_cell_config::Location", tags = "1, 2")]
    #[serde(flatten)]
    pub location: ::std::option::Option<node_cell_config::Location>,
}
pub mod node_cell_config {
    #[derive(Clone, PartialEq, ::prost::Oneof, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Location {
        #[prost(message, tag = "1")]
        Inline(super::CellConfig),
        #[prost(string, tag = "2")]
        Path(std::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeStoreConfig {
    #[prost(message, optional, tag = "1")]
    #[serde(default)]
    pub index: ::std::option::Option<EntityIndexConfig>,
}
//// Configuration of the entities index
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct EntityIndexConfig {
    #[prost(uint64, tag = "1")]
    pub chain_index_min_depth: u64,
    #[prost(uint64, tag = "2")]
    pub chain_index_depth_leeway: u64,
    #[prost(message, optional, tag = "3")]
    pub pending_index: ::std::option::Option<MutationIndexConfig>,
    #[prost(message, optional, tag = "4")]
    pub chain_index: ::std::option::Option<MutationIndexConfig>,
}
//// Trait index configuration
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellConfig {
    #[prost(string, tag = "1")]
    pub public_key: std::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub keypair: std::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub name: std::string::String,
    #[prost(string, tag = "4")]
    #[serde(default)]
    pub id: std::string::String,
    #[prost(string, tag = "5")]
    #[serde(default)]
    pub path: std::string::String,
    #[prost(message, repeated, tag = "6")]
    pub nodes: ::std::vec::Vec<CellNodeConfig>,
    #[prost(message, repeated, tag = "7")]
    #[serde(default)]
    pub apps: ::std::vec::Vec<CellApplicationConfig>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellNodeConfig {
    #[prost(message, optional, tag = "1")]
    pub node: ::std::option::Option<NodeConfig>,
    #[prost(enumeration = "cell_node_config::Role", repeated, tag = "2")]
    #[serde(default)]
    pub roles: ::std::vec::Vec<i32>,
}
pub mod cell_node_config {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(Serialize, Deserialize)]
    pub enum Role {
        InvalidRole = 0,
        ChainRole = 1,
        StoreRole = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeConfig {
    #[prost(string, tag = "1")]
    pub public_key: std::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub name: std::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub id: std::string::String,
    #[prost(message, optional, tag = "4")]
    pub addresses: ::std::option::Option<NodeAddresses>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellApplicationConfig {
    #[prost(oneof = "cell_application_config::Location", tags = "1, 2")]
    #[serde(flatten)]
    pub location: ::std::option::Option<cell_application_config::Location>,
}
pub mod cell_application_config {
    #[derive(Clone, PartialEq, ::prost::Oneof, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Location {
        #[prost(message, tag = "1")]
        Inline(super::super::apps::Manifest),
        #[prost(string, tag = "2")]
        Path(std::string::String),
    }
}
