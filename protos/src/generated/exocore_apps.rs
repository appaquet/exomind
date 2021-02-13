#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct Manifest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    #[serde(default)]
    pub path: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    #[serde(default)]
    pub schemas: ::prost::alloc::vec::Vec<ManifestSchema>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct ManifestSchema {
    #[prost(oneof = "manifest_schema::Source", tags = "1, 2")]
    #[serde(flatten)]
    pub source: ::core::option::Option<manifest_schema::Source>,
}
/// Nested message and enum types in `ManifestSchema`.
pub mod manifest_schema {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Source {
        #[prost(string, tag = "1")]
        File(::prost::alloc::string::String),
        #[prost(bytes, tag = "2")]
        #[serde(
            serialize_with = "crate::base64::as_base64",
            deserialize_with = "crate::base64::from_base64"
        )]
        Bytes(::prost::alloc::vec::Vec<u8>),
    }
}
