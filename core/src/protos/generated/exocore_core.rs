#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct LocalNodeConfig {
    #[prost(string, tag = "1")]
    pub keypair: std::string::String,
    #[prost(string, tag = "2")]
    pub public_key: std::string::String,
    #[prost(message, repeated, tag = "3")]
    pub cells: ::std::vec::Vec<CellConfig>,
    #[prost(string, repeated, tag = "4")]
    pub listen_addresses: ::std::vec::Vec<std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct NodeConfig {
    #[prost(string, tag = "1")]
    pub public_key: std::string::String,
    #[prost(string, repeated, tag = "2")]
    pub addresses: ::std::vec::Vec<std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellConfig {
    #[prost(string, tag = "1")]
    pub public_key: std::string::String,
    #[prost(string, tag = "2")]
    pub keypair: std::string::String,
    #[prost(string, tag = "3")]
    pub data_directory: std::string::String,
    #[prost(message, repeated, tag = "4")]
    pub nodes: ::std::vec::Vec<CellNodeConfig>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct CellNodeConfig {
    #[prost(message, optional, tag = "1")]
    pub node: ::std::option::Option<NodeConfig>,
    #[prost(enumeration = "cell_node_config::Role", repeated, tag = "2")]
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
