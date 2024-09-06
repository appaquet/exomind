#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMessage {
    #[prost(string, tag = "1")]
    pub string1: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub string2: ::prost::alloc::string::String,
    #[prost(string, tag = "12")]
    pub string3: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub struct1: ::core::option::Option<TestStruct>,
    #[prost(message, optional, tag = "8")]
    pub date1: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "9")]
    pub date2: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "17")]
    pub date3: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint32, tag = "10")]
    pub uint1: u32,
    #[prost(uint32, tag = "11")]
    pub uint2: u32,
    #[prost(uint32, tag = "18")]
    pub uint3: u32,
    #[prost(int32, tag = "15")]
    pub int1: i32,
    #[prost(int32, tag = "16")]
    pub int2: i32,
    #[prost(int32, tag = "19")]
    pub int3: i32,
    #[prost(message, optional, tag = "13")]
    pub ref1: ::core::option::Option<super::store::Reference>,
    #[prost(message, optional, tag = "14")]
    pub ref2: ::core::option::Option<super::store::Reference>,
    #[prost(string, tag = "20")]
    pub grouped1: ::prost::alloc::string::String,
    #[prost(string, tag = "21")]
    pub grouped2: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "22")]
    pub map1:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(oneof = "test_message::Fields", tags = "4, 5")]
    pub fields: ::core::option::Option<test_message::Fields>,
}
/// Nested message and enum types in `TestMessage`.
pub mod test_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Fields {
        #[prost(string, tag = "4")]
        OneofString1(::prost::alloc::string::String),
        #[prost(uint32, tag = "5")]
        OneofInt1(u32),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestStruct {
    #[prost(string, tag = "1")]
    pub string1: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMessage2 {
    #[prost(string, tag = "1")]
    pub string1: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub string2: ::prost::alloc::string::String,
}
