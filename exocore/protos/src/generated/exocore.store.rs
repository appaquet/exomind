#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entity {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub traits: ::prost::alloc::vec::Vec<Trait>,
    #[prost(message, optional, tag = "5")]
    pub creation_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub modification_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "7")]
    pub deletion_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag = "8")]
    pub last_operation_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trait {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub message: ::core::option::Option<::prost_types::Any>,
    #[prost(message, optional, tag = "3")]
    pub creation_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub modification_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub deletion_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(uint64, tag = "7")]
    pub last_operation_id: u64,
    #[prost(enumeration = "TraitDetails", tag = "5")]
    pub details: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reference {
    #[prost(string, tag = "1")]
    pub entity_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub trait_id: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TraitDetails {
    Full = 0,
    Partial = 1,
}
impl TraitDetails {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic
    /// use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TraitDetails::Full => "TRAIT_DETAILS_FULL",
            TraitDetails::Partial => "TRAIT_DETAILS_PARTIAL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TRAIT_DETAILS_FULL" => Some(Self::Full),
            "TRAIT_DETAILS_PARTIAL" => Some(Self::Partial),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityQuery {
    /// Optional projections on traits and fields to be returned.
    #[prost(message, repeated, tag = "7")]
    pub projections: ::prost::alloc::vec::Vec<Projection>,
    /// Query paging requested.
    #[prost(message, optional, tag = "5")]
    pub paging: ::core::option::Option<Paging>,
    /// Query ordering.
    #[prost(message, optional, tag = "6")]
    pub ordering: ::core::option::Option<Ordering>,
    /// Optional watch token if this query is to be used for watching.
    #[prost(uint64, tag = "8")]
    pub watch_token: u64,
    /// If specified, if results from server matches this hash, results will be
    /// empty with the `skipped_hash` field set to `true`.
    #[prost(uint64, tag = "9")]
    pub result_hash: u64,
    /// Include deleted mutations matches. This can be used to return recently
    /// modified entities that also include deletions. Deleted traits will
    /// be included in the results but will have a `deletion_date` field
    /// with the date of the deletion.
    #[prost(bool, tag = "12")]
    pub include_deleted: bool,
    /// Indicates that the query comes from an automated / programmatic logic.
    /// This is used since chain indexation may be deferred until no user
    /// queries got received for a while.
    #[prost(bool, tag = "13")]
    pub programmatic: bool,
    /// Main search predicate on individual traits of the entity.
    #[prost(
        oneof = "entity_query::Predicate",
        tags = "1, 2, 3, 4, 10, 11, 14, 15, 99"
    )]
    pub predicate: ::core::option::Option<entity_query::Predicate>,
}
/// Nested message and enum types in `EntityQuery`.
pub mod entity_query {
    /// Main search predicate on individual traits of the entity.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Predicate {
        #[prost(message, tag = "1")]
        Match(super::MatchPredicate),
        #[prost(message, tag = "2")]
        Trait(super::TraitPredicate),
        #[prost(message, tag = "3")]
        Ids(super::IdsPredicate),
        #[prost(message, tag = "4")]
        Reference(super::ReferencePredicate),
        #[prost(message, tag = "10")]
        Operations(super::OperationsPredicate),
        #[prost(message, tag = "11")]
        All(super::AllPredicate),
        #[prost(message, tag = "14")]
        Boolean(super::BooleanPredicate),
        #[prost(message, tag = "15")]
        QueryString(super::QueryStringPredicate),
        #[prost(message, tag = "99")]
        Test(super::TestPredicate),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Projection {
    /// If specified, a prefix match will be done against traits' Protobuf full
    /// name (`some.package.Name`). If ends with a dollar sign "$", an exact
    /// match is required (ex: `some.package.Name$` will only match this
    /// message)
    #[prost(string, repeated, tag = "1")]
    pub package: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Skips the trait if the projection matches.
    #[prost(bool, tag = "2")]
    pub skip: bool,
    /// If specified, only return these fields.
    #[prost(uint32, repeated, tag = "4")]
    pub field_ids: ::prost::alloc::vec::Vec<u32>,
    /// If specified, only return fields annotated with
    /// `options.proto`.`field_group_id` matching ids.
    #[prost(uint32, repeated, tag = "5")]
    pub field_group_ids: ::prost::alloc::vec::Vec<u32>,
}
/// Query entities by text match on all indexed fields across all traits.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MatchPredicate {
    /// Text query.
    #[prost(string, tag = "1")]
    pub query: ::prost::alloc::string::String,
    /// Disable fuzzy matching.
    #[prost(bool, tag = "2")]
    pub no_fuzzy: bool,
}
/// Query entities by IDs.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdsPredicate {
    #[prost(string, repeated, tag = "1")]
    pub ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Query entities by mutations' operation ids.
/// Used to return entities on which mutations with these operation ids were
/// applied and indexed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperationsPredicate {
    #[prost(uint64, repeated, tag = "1")]
    pub operation_ids: ::prost::alloc::vec::Vec<u64>,
}
/// Query all entities.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllPredicate {}
/// Used for tests.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestPredicate {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
/// Boolean query constructed of different sub-queries with boolean operators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BooleanPredicate {
    #[prost(message, repeated, tag = "1")]
    pub queries: ::prost::alloc::vec::Vec<boolean_predicate::SubQuery>,
}
/// Nested message and enum types in `BooleanPredicate`.
pub mod boolean_predicate {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SubQuery {
        #[prost(enumeration = "Occur", tag = "1")]
        pub occur: i32,
        #[prost(oneof = "sub_query::Predicate", tags = "2, 3, 4, 5, 6, 7, 8")]
        pub predicate: ::core::option::Option<sub_query::Predicate>,
    }
    /// Nested message and enum types in `SubQuery`.
    pub mod sub_query {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Predicate {
            #[prost(message, tag = "2")]
            Match(super::super::MatchPredicate),
            #[prost(message, tag = "3")]
            Trait(super::super::TraitPredicate),
            #[prost(message, tag = "4")]
            Ids(super::super::IdsPredicate),
            #[prost(message, tag = "5")]
            Reference(super::super::ReferencePredicate),
            #[prost(message, tag = "6")]
            Operations(super::super::OperationsPredicate),
            #[prost(message, tag = "7")]
            All(super::super::AllPredicate),
            #[prost(message, tag = "8")]
            Boolean(super::super::BooleanPredicate),
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Occur {
        Should = 0,
        Must = 1,
        MustNot = 2,
    }
    impl Occur {
        /// String value of the enum field names used in the ProtoBuf
        /// definition.
        ///
        /// The values are not transformed in any way and thus are considered
        /// stable (if the ProtoBuf definition does not change) and safe
        /// for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Occur::Should => "SHOULD",
                Occur::Must => "MUST",
                Occur::MustNot => "MUST_NOT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "SHOULD" => Some(Self::Should),
                "MUST" => Some(Self::Must),
                "MUST_NOT" => Some(Self::MustNot),
                _ => None,
            }
        }
    }
}
/// Query entities that have a specified trait and optionally matching a trait
/// query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitPredicate {
    #[prost(string, tag = "1")]
    pub trait_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub query: ::core::option::Option<TraitQuery>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitQuery {
    #[prost(oneof = "trait_query::Predicate", tags = "1, 2, 3, 4")]
    pub predicate: ::core::option::Option<trait_query::Predicate>,
}
/// Nested message and enum types in `TraitQuery`.
pub mod trait_query {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Predicate {
        #[prost(message, tag = "1")]
        Match(super::MatchPredicate),
        #[prost(message, tag = "2")]
        Field(super::TraitFieldPredicate),
        #[prost(message, tag = "3")]
        Reference(super::TraitFieldReferencePredicate),
        #[prost(message, tag = "4")]
        QueryString(super::QueryStringPredicate),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitFieldPredicate {
    #[prost(string, tag = "1")]
    pub field: ::prost::alloc::string::String,
    #[prost(enumeration = "trait_field_predicate::Operator", tag = "6")]
    pub operator: i32,
    #[prost(oneof = "trait_field_predicate::Value", tags = "2, 3, 4, 5")]
    pub value: ::core::option::Option<trait_field_predicate::Value>,
}
/// Nested message and enum types in `TraitFieldPredicate`.
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
    impl Operator {
        /// String value of the enum field names used in the ProtoBuf
        /// definition.
        ///
        /// The values are not transformed in any way and thus are considered
        /// stable (if the ProtoBuf definition does not change) and safe
        /// for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Operator::Equal => "EQUAL",
                Operator::Gt => "GT",
                Operator::Gte => "GTE",
                Operator::Lt => "LT",
                Operator::Lte => "LTE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "EQUAL" => Some(Self::Equal),
                "GT" => Some(Self::Gt),
                "GTE" => Some(Self::Gte),
                "LT" => Some(Self::Lt),
                "LTE" => Some(Self::Lte),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "2")]
        String(::prost::alloc::string::String),
        #[prost(int64, tag = "3")]
        Int64(i64),
        #[prost(uint64, tag = "4")]
        Uint64(u64),
        #[prost(message, tag = "5")]
        Date(::prost_types::Timestamp),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraitFieldReferencePredicate {
    #[prost(string, tag = "1")]
    pub field: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub reference: ::core::option::Option<ReferencePredicate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReferencePredicate {
    /// Entity id the reference points to
    #[prost(string, tag = "1")]
    pub entity_id: ::prost::alloc::string::String,
    /// Optional trait id the reference points to
    #[prost(string, tag = "2")]
    pub trait_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryStringPredicate {
    #[prost(string, tag = "1")]
    pub query: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Paging {
    /// Returns results after this given ordering value.
    #[prost(message, optional, tag = "1")]
    pub after_ordering_value: ::core::option::Option<OrderingValue>,
    /// Returns results before this given ordering value.
    #[prost(message, optional, tag = "2")]
    pub before_ordering_value: ::core::option::Option<OrderingValue>,
    /// Desired results count. Default if 0.
    #[prost(uint32, tag = "3")]
    pub count: u32,
    /// Mutation index use only, no effect on entity query.
    #[prost(uint32, tag = "4")]
    pub offset: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ordering {
    /// Direction of ordering.
    #[prost(bool, tag = "4")]
    pub ascending: bool,
    /// If match score used, don't boost recent documents
    #[prost(bool, tag = "5")]
    pub no_recency_boost: bool,
    /// If match score used, don't boost results that have references.
    #[prost(bool, tag = "6")]
    pub no_reference_boost: bool,
    /// Value by which we want results to be ordered.
    #[prost(oneof = "ordering::Value", tags = "1, 2, 3, 7, 8")]
    pub value: ::core::option::Option<ordering::Value>,
}
/// Nested message and enum types in `Ordering`.
pub mod ordering {
    /// Value by which we want results to be ordered.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// by match score + date boost
        #[prost(bool, tag = "1")]
        Score(bool),
        /// by operation id
        #[prost(bool, tag = "2")]
        OperationId(bool),
        /// by field value
        #[prost(string, tag = "3")]
        Field(::prost::alloc::string::String),
        /// by creation date
        #[prost(bool, tag = "7")]
        CreatedAt(bool),
        /// by update date
        #[prost(bool, tag = "8")]
        UpdatedAt(bool),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderingValue {
    /// Secondary comparison, in case values were equal. In this case,
    /// the last operation id that mutated the entity is used.
    #[prost(uint64, tag = "6")]
    pub operation_id: u64,
    /// Primary comparison
    #[prost(oneof = "ordering_value::Value", tags = "1, 2, 3, 4, 5")]
    pub value: ::core::option::Option<ordering_value::Value>,
}
/// Nested message and enum types in `OrderingValue`.
pub mod ordering_value {
    /// Primary comparison
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(float, tag = "1")]
        Float(f32),
        #[prost(uint64, tag = "2")]
        Uint64(u64),
        #[prost(message, tag = "3")]
        Date(::prost_types::Timestamp),
        #[prost(bool, tag = "4")]
        Min(bool),
        #[prost(bool, tag = "5")]
        Max(bool),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityResults {
    /// Entities matching query.
    #[prost(message, repeated, tag = "1")]
    pub entities: ::prost::alloc::vec::Vec<EntityResult>,
    /// If query specified a `result_hash`, this is set to `true` if the results
    /// had the same hash has the specified and that `entities` were set to
    /// empty.
    #[prost(bool, tag = "2")]
    pub skipped_hash: bool,
    /// Estimated number of entities matching, based on number of matching
    /// mutations.
    #[prost(uint32, tag = "3")]
    pub estimated_count: u32,
    /// Paging token of the current results.
    #[prost(message, optional, tag = "4")]
    pub current_page: ::core::option::Option<Paging>,
    /// Paging token of the next page of results.
    #[prost(message, optional, tag = "5")]
    pub next_page: ::core::option::Option<Paging>,
    /// Hash of the results. Can be used to prevent receiving same results if
    /// they haven't changed by using the `result_hash` field on the query.
    #[prost(uint64, tag = "6")]
    pub hash: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityResult {
    /// The entity with its traits. Projection could have been done on the
    /// entity, which will be indicated in its traits' details field.
    #[prost(message, optional, tag = "1")]
    pub entity: ::core::option::Option<Entity>,
    /// Indicates where the entity was taken from in terms of storage. If all of
    /// the entities' traits were coming from the chain (i.e. committed),
    /// the source will be `CHAIN`. Otherwise, as soon as one entity
    /// mutation is coming from pending store (i.e. not committed yet), this
    /// field will be `PENDING`.
    ///
    /// This can be used to know if an entity can be considered stable once
    /// mutations were executed on it. Once it's committed, a majority of
    /// nodes agreed on it and will not result in further changes happening
    /// before the latest consistent timestamp.
    #[prost(enumeration = "EntityResultSource", tag = "2")]
    pub source: i32,
    /// Value to be used to order results. `EntityResults` already contains
    /// ordered results, but it may be useful to compare ordering queries
    /// (ex.: to merge different pages)
    #[prost(message, optional, tag = "3")]
    pub ordering_value: ::core::option::Option<OrderingValue>,
    /// Hash of the entity result. Can be used to compare if the entity has
    /// changed since last results.
    #[prost(uint64, tag = "4")]
    pub hash: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EntityResultSource {
    Unknown = 0,
    Pending = 1,
    Chain = 2,
}
impl EntityResultSource {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic
    /// use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            EntityResultSource::Unknown => "UNKNOWN",
            EntityResultSource::Pending => "PENDING",
            EntityResultSource::Chain => "CHAIN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "PENDING" => Some(Self::Pending),
            "CHAIN" => Some(Self::Chain),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutationRequest {
    /// Mutations to apply.
    #[prost(message, repeated, tag = "1")]
    pub mutations: ::prost::alloc::vec::Vec<EntityMutation>,
    /// Waits for mutation to be indexed.
    #[prost(bool, tag = "2")]
    pub wait_indexed: bool,
    /// Waits for mutation to be indexed and returns the mutated entities.
    #[prost(bool, tag = "3")]
    pub return_entities: bool,
    /// If an entity ID is generated for the mutated entities, reuse the same ID
    /// for all mutations.
    #[prost(bool, tag = "4")]
    pub common_entity_id: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutationResult {
    /// Unique operation ids for each mutations.
    #[prost(uint64, repeated, tag = "1")]
    pub operation_ids: ::prost::alloc::vec::Vec<u64>,
    /// Mutated entities if requested.
    #[prost(message, repeated, tag = "2")]
    pub entities: ::prost::alloc::vec::Vec<Entity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityMutation {
    #[prost(string, tag = "1")]
    pub entity_id: ::prost::alloc::string::String,
    #[prost(oneof = "entity_mutation::Mutation", tags = "2, 3, 4, 7, 99")]
    pub mutation: ::core::option::Option<entity_mutation::Mutation>,
}
/// Nested message and enum types in `EntityMutation`.
pub mod entity_mutation {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Mutation {
        #[prost(message, tag = "2")]
        PutTrait(super::PutTraitMutation),
        #[prost(message, tag = "3")]
        DeleteTrait(super::DeleteTraitMutation),
        #[prost(message, tag = "4")]
        DeleteEntity(super::DeleteEntityMutation),
        #[prost(message, tag = "7")]
        DeleteOperations(super::DeleteOperationsMutation),
        #[prost(message, tag = "99")]
        Test(super::TestMutation),
    }
}
/// Creates or overrides a trait of the entity.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutTraitMutation {
    #[prost(message, optional, tag = "1")]
    pub r#trait: ::core::option::Option<Trait>,
}
/// Deletes a trait of an entity.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTraitMutation {
    #[prost(string, tag = "1")]
    pub trait_id: ::prost::alloc::string::String,
}
/// Deletes all the traits of an entity, effectively deleting the entity itself.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteEntityMutation {}
/// Deletes mutations of an entity that have an operation id specified in the
/// given list. This mutation is used for index management purpose only since
/// the mutations are not actually deleted from the chain. Since the chain
/// indices are built in a semi-versioned way, this actually delete the
/// mutations from the indices.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteOperationsMutation {
    #[prost(uint64, repeated, tag = "1")]
    pub operation_ids: ::prost::alloc::vec::Vec<u64>,
}
/// Mutation used in tests.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestMutation {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
/// Internal message used by entity iterator for external sorting.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommittedEntityMutation {
    #[prost(uint64, tag = "1")]
    pub block_offset: u64,
    #[prost(uint64, tag = "2")]
    pub operation_id: u64,
    #[prost(message, optional, tag = "3")]
    pub mutation: ::core::option::Option<EntityMutation>,
}
