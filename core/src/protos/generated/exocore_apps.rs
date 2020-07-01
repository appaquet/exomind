#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct Manifest {
    #[prost(string, tag = "1")]
    pub name: std::string::String,
    #[prost(string, tag = "2")]
    pub public_key: std::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub path: std::string::String,
    #[prost(message, repeated, tag = "4")]
    #[serde(default)]
    pub schemas: ::std::vec::Vec<ManifestSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct ManifestSchema {
    #[prost(oneof = "manifest_schema::Source", tags = "1, 2")]
    pub source: ::std::option::Option<manifest_schema::Source>,
}
pub mod manifest_schema {
    #[derive(Clone, PartialEq, ::prost::Oneof, Serialize, Deserialize)]
    pub enum Source {
        #[prost(string, tag = "1")]
        File(std::string::String),
        #[prost(bytes, tag = "2")]
        Bytes(std::vec::Vec<u8>),
    }
}
