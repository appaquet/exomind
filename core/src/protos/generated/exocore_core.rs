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
    pub path: std::string::String,
    #[prost(string, repeated, tag = "5")]
    pub listen_addresses: ::std::vec::Vec<std::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub cells: ::std::vec::Vec<NodeCellConfig>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeCellConfig {
    #[prost(oneof = "node_cell_config::Location", tags = "1, 2")]
    pub location: ::std::option::Option<node_cell_config::Location>,
}
pub mod node_cell_config {
    #[derive(Clone, PartialEq, ::prost::Oneof, Serialize, Deserialize)]
    pub enum Location {
        #[prost(message, tag = "1")]
        Instance(super::CellConfig),
        #[prost(string, tag = "2")]
        Directory(std::string::String),
    }
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
    pub path: std::string::String,
    #[prost(message, repeated, tag = "5")]
    pub nodes: ::std::vec::Vec<CellNodeConfig>,
    #[prost(message, repeated, tag = "6")]
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
        DataRole = 1,
        IndexStoreRole = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeConfig {
    #[prost(string, tag = "1")]
    pub public_key: std::string::String,
    #[prost(string, repeated, tag = "2")]
    #[serde(default)]
    pub addresses: ::std::vec::Vec<std::string::String>,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellApplicationConfig {
    #[prost(oneof = "cell_application_config::Location", tags = "1, 2")]
    pub location: ::std::option::Option<cell_application_config::Location>,
}
pub mod cell_application_config {
    #[derive(Clone, PartialEq, ::prost::Oneof, Serialize, Deserialize)]
    pub enum Location {
        #[prost(message, tag = "1")]
        Instance(super::super::apps::Manifest),
        #[prost(string, tag = "2")]
        Directory(std::string::String),
    }
}
