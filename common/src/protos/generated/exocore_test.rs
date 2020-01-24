#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMessage {
    #[prost(string, tag = "1")]
    pub string1: std::string::String,
    #[prost(string, tag = "2")]
    pub string2: std::string::String,
    #[prost(message, optional, tag = "3")]
    pub struct1: ::std::option::Option<TestStruct>,
    #[prost(message, optional, tag = "8")]
    pub date1: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "9")]
    pub date2: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(uint32, tag = "10")]
    pub int1: u32,
    #[prost(uint32, tag = "11")]
    pub int2: u32,
    #[prost(oneof = "test_message::Fields", tags = "4, 5")]
    pub fields: ::std::option::Option<test_message::Fields>,
}
pub mod test_message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Fields {
        #[prost(string, tag = "4")]
        OneofString1(std::string::String),
        #[prost(uint32, tag = "5")]
        OneofInt1(u32),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestStruct {
    #[prost(string, tag = "1")]
    pub string1: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMessage2 {
    #[prost(string, tag = "1")]
    pub string1: std::string::String,
    #[prost(string, tag = "2")]
    pub string2: std::string::String,
}
