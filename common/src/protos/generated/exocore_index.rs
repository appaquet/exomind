#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entity {
    #[prost(string, tag = "1")]
    pub id: std::string::String,
    #[prost(message, repeated, tag = "4")]
    pub traits: ::std::vec::Vec<Trait>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trait {
    #[prost(message, optional, tag = "1")]
    pub message: ::std::option::Option<::prost_types::Any>,
    #[prost(message, optional, tag = "2")]
    pub created_at: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub modified_at: ::std::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityQuery {
    #[prost(message, optional, tag = "5")]
    pub paging: ::std::option::Option<Paging>,
    //// If true, only return summary
    #[prost(bool, tag = "6")]
    pub summary: bool,
    //// Optional watch token if this query is to be used for watching.
    #[prost(string, tag = "7")]
    pub watch_token: std::string::String,
    //// If specified, if results from server matches this hash, only a summary will be returned.
    #[prost(string, tag = "8")]
    pub result_hash: std::string::String,
    #[prost(oneof = "entity_query::Predicate", tags = "1, 2, 3, 4")]
    pub predicate: ::std::option::Option<entity_query::Predicate>,
}
pub mod entity_query {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Predicate {
        #[prost(message, tag = "1")]
        Match(super::MatchPredicate),
        #[prost(message, tag = "2")]
        Trait(super::TraitPredicate),
        #[prost(message, tag = "3")]
        Id(super::IdPredicate),
        #[prost(message, tag = "4")]
        Fail(super::FailPredicate),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MatchPredicate {
    #[prost(string, tag = "1")]
    pub query: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdPredicate {
    #[prost(string, tag = "1")]
    pub id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FailPredicate {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitPredicate {
    #[prost(string, tag = "1")]
    pub trait_name: std::string::String,
    #[prost(message, optional, tag = "2")]
    pub query: ::std::option::Option<TraitQuery>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitQuery {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Paging {
    //// Returns results after token.
    #[prost(string, tag = "1")]
    pub after_token: std::string::String,
    //// Returns results before token.
    #[prost(string, tag = "2")]
    pub before_token: std::string::String,
    //// Desired results count. Default if 0.
    #[prost(uint32, tag = "3")]
    pub count: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityResults {
    #[prost(message, repeated, tag = "1")]
    pub entities: ::std::vec::Vec<EntityResults>,
    #[prost(bool, tag = "2")]
    pub summary: bool,
    #[prost(uint32, tag = "3")]
    pub estimated_count: u32,
    #[prost(message, optional, tag = "4")]
    pub current_page: ::std::option::Option<Paging>,
    #[prost(message, optional, tag = "5")]
    pub next_page: ::std::option::Option<Paging>,
    #[prost(string, tag = "6")]
    pub hash: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityResult {
    #[prost(message, optional, tag = "1")]
    pub entity: ::std::option::Option<Entity>,
    #[prost(enumeration = "EntityResultSource", tag = "2")]
    pub source: i32,
    #[prost(string, tag = "3")]
    pub sort_token: std::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EntityResultSource {
    Unknown = 0,
    Pending = 1,
    Chain = 2,
}
