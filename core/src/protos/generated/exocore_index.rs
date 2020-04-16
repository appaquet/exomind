#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entity {
    #[prost(string, tag = "1")]
    pub id: std::string::String,
    #[prost(message, repeated, tag = "4")]
    pub traits: ::std::vec::Vec<Trait>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trait {
    #[prost(string, tag = "1")]
    pub id: std::string::String,
    #[prost(message, optional, tag = "2")]
    pub message: ::std::option::Option<::prost_types::Any>,
    #[prost(message, optional, tag = "3")]
    pub creation_date: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub modification_date: ::std::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reference {
    #[prost(string, tag = "1")]
    pub entity_id: std::string::String,
    #[prost(string, tag = "2")]
    pub trait_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityQuery {
    //// Query paging requested
    #[prost(message, optional, tag = "5")]
    pub paging: ::std::option::Option<Paging>,
    //// Query sorting
    #[prost(message, optional, tag = "6")]
    pub sorting: ::std::option::Option<Sorting>,
    //// If true, only return summary
    #[prost(bool, tag = "7")]
    pub summary: bool,
    //// Optional watch token if this query is to be used for watching.
    #[prost(uint64, tag = "8")]
    pub watch_token: u64,
    //// If specified, if results from server matches this hash, only a summary will be returned.
    #[prost(uint64, tag = "9")]
    pub result_hash: u64,
    #[prost(oneof = "entity_query::Predicate", tags = "1, 2, 3, 4, 99")]
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
        Reference(super::ReferencePredicate),
        #[prost(message, tag = "99")]
        Test(super::TestPredicate),
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
pub struct TestPredicate {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitPredicate {
    #[prost(string, tag = "1")]
    pub trait_name: std::string::String,
    #[prost(message, optional, tag = "2")]
    pub query: ::std::option::Option<TraitQuery>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitQuery {
    #[prost(oneof = "trait_query::Query", tags = "1, 2, 3")]
    pub query: ::std::option::Option<trait_query::Query>,
}
pub mod trait_query {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Query {
        #[prost(message, tag = "1")]
        Reference(super::ReferencePredicate),
        #[prost(message, tag = "2")]
        Match(super::MatchPredicate),
        #[prost(message, tag = "3")]
        Field(super::TraitFieldPredicate),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitFieldPredicate {
    #[prost(string, tag = "1")]
    pub field: std::string::String,
    #[prost(enumeration = "trait_field_predicate::Operator", tag = "6")]
    pub operatior: i32,
    #[prost(oneof = "trait_field_predicate::Value", tags = "2, 3, 4, 5")]
    pub value: ::std::option::Option<trait_field_predicate::Value>,
}
pub mod trait_field_predicate {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operator {
        Equal = 0,
        Gt = 1,
        Gte = 2,
        Lt = 3,
        Lte = 4,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "2")]
        String(std::string::String),
        #[prost(int64, tag = "3")]
        Int64(i64),
        #[prost(uint64, tag = "4")]
        Uint64(u64),
        #[prost(message, tag = "5")]
        Date(::prost_types::Timestamp),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReferencePredicate {
    /// Entity id the reference points to
    #[prost(string, tag = "1")]
    pub entity_id: std::string::String,
    /// Optional trait id the reference points to
    #[prost(string, tag = "2")]
    pub trait_id: std::string::String,
}
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
pub struct Sorting {
    #[prost(bool, tag = "4")]
    pub ascending: bool,
    #[prost(oneof = "sorting::Value", tags = "1, 2, 3")]
    pub value: ::std::option::Option<sorting::Value>,
}
pub mod sorting {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(bool, tag = "1")]
        Score(bool),
        #[prost(bool, tag = "2")]
        OperationId(bool),
        #[prost(string, tag = "3")]
        Field(std::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityResults {
    #[prost(message, repeated, tag = "1")]
    pub entities: ::std::vec::Vec<EntityResult>,
    #[prost(bool, tag = "2")]
    pub summary: bool,
    #[prost(uint32, tag = "3")]
    pub estimated_count: u32,
    #[prost(message, optional, tag = "4")]
    pub current_page: ::std::option::Option<Paging>,
    #[prost(message, optional, tag = "5")]
    pub next_page: ::std::option::Option<Paging>,
    #[prost(uint64, tag = "6")]
    pub hash: u64,
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityMutation {
    #[prost(string, tag = "1")]
    pub entity_id: std::string::String,
    #[prost(oneof = "entity_mutation::Mutation", tags = "2, 3, 4, 5, 6, 99")]
    pub mutation: ::std::option::Option<entity_mutation::Mutation>,
}
pub mod entity_mutation {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Mutation {
        #[prost(message, tag = "2")]
        PutTrait(super::PutTraitMutation),
        #[prost(message, tag = "3")]
        DeleteTrait(super::DeleteTraitMutation),
        #[prost(message, tag = "4")]
        DeleteEntity(super::DeleteEntityMutation),
        #[prost(message, tag = "5")]
        UpdateTrait(super::UpdateTraitMutation),
        #[prost(message, tag = "6")]
        CompactTrait(super::CompactTraitMutation),
        #[prost(message, tag = "99")]
        Test(super::TestMutation),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutTraitMutation {
    #[prost(message, optional, tag = "1")]
    pub r#trait: ::std::option::Option<Trait>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTraitMutation {
    #[prost(string, tag = "1")]
    pub trait_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteEntityMutation {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateTraitMutation {
    #[prost(string, tag = "1")]
    pub trait_id: std::string::String,
    #[prost(message, optional, tag = "2")]
    pub r#trait: ::std::option::Option<Trait>,
    #[prost(message, optional, tag = "3")]
    pub field_mask: ::std::option::Option<::prost_types::FieldMask>,
    /// Updates is only valid if the last mutation operation on trait this given operation id.
    #[prost(uint64, tag = "4")]
    pub if_last_operation_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactTraitMutation {
    /// List of operations that are compacted by this compaction. The compaction will only succeed
    /// if there were no operations between these operations and the compaction's operation itself.
    #[prost(message, repeated, tag = "1")]
    pub compacted_operations: ::std::vec::Vec<compact_trait_mutation::Operation>,
    /// Trait with merged values from compacted operations
    #[prost(message, optional, tag = "2")]
    pub r#trait: ::std::option::Option<Trait>,
}
pub mod compact_trait_mutation {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Operation {
        #[prost(uint64, tag = "1")]
        pub operation_id: u64,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMutation {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutationResult {
    #[prost(uint64, tag = "1")]
    pub operation_id: u64,
}
