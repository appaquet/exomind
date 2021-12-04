#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct Manifest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    #[serde(default)]
    pub schemas: ::prost::alloc::vec::Vec<ManifestSchema>,
    #[prost(message, optional, tag = "5")]
    pub module: ::core::option::Option<ManifestModule>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct ManifestSchema {
    #[prost(oneof = "manifest_schema::Source", tags = "1")]
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
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct ManifestModule {
    #[prost(string, tag = "1")]
    pub file: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub multihash: ::prost::alloc::string::String,
}
/// Message sent to application running in WASM from runtime.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InMessage {
    #[prost(enumeration = "in_message::InMessageType", tag = "1")]
    pub r#type: i32,
    /// if message is a response to a previous outgoing message, this identifier
    /// will be the same as the outgoing message
    #[prost(uint32, tag = "2")]
    pub rendez_vous_id: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "4")]
    pub error: ::prost::alloc::string::String,
}
/// Nested message and enum types in `InMessage`.
pub mod in_message {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum InMessageType {
        Invalid = 0,
        StoreEntityResults = 1,
        StoreMutationResult = 2,
    }
}
/// Message sent from application running in WASM to runtime.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutMessage {
    #[prost(enumeration = "out_message::OutMessageType", tag = "1")]
    pub r#type: i32,
    /// if message require a response, id that will be used to match
    /// response back to callee
    #[prost(uint32, tag = "2")]
    pub rendez_vous_id: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `OutMessage`.
pub mod out_message {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OutMessageType {
        Invalid = 0,
        StoreEntityQuery = 1,
        StoreMutationRequest = 2,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MessageStatus {
    Ok = 0,
    Unhandled = 1,
    DecodeError = 2,
}
