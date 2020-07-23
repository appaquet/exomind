use prost::Message;

use exocore_chain::operation::OperationId;
use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::exocore_index::{
    entity_query, ordering, trait_field_predicate, trait_query, EntityQuery, EntityResults,
    MatchPredicate, Ordering, Paging, ReferencePredicate, TestPredicate, TraitFieldPredicate,
    TraitFieldReferencePredicate, TraitPredicate, TraitQuery,
};
use exocore_core::protos::generated::index_transport_capnp::watched_query_request;
use exocore_core::protos::generated::index_transport_capnp::{query_request, query_response};
use exocore_core::protos::{
    index::{AllPredicate, IdsPredicate, OperationsPredicate, Projection},
    message::NamedMessage,
    prost::ProstMessageExt,
    reflect::FieldId,
};

use crate::entity::{EntityId, TraitId};
use crate::error::Error;

pub type WatchToken = u64;
pub type ResultHash = u64;

#[derive(Clone)]
pub struct QueryBuilder {
    query: EntityQuery,
}

impl QueryBuilder {
    pub fn matches<T: Into<String>>(query: T) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Match(MatchPredicate {
                    query: query.into(),
                })),
                ..Default::default()
            },
        }
    }

    pub fn references<T: Into<ReferencePredicateWrapper>>(reference: T) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Reference(reference.into().0)),
                ..Default::default()
            },
        }
    }

    pub fn with_operations(operation_ids: Vec<OperationId>) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Operations(OperationsPredicate {
                    operation_ids,
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_trait_name<T: Into<String>>(trait_name: T) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Trait(TraitPredicate {
                    trait_name: trait_name.into(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_trait<T: NamedMessage>() -> QueryBuilder {
        Self::with_trait_name(T::full_name())
    }

    pub fn with_trait_name_query<T: Into<String>>(
        trait_name: T,
        query: TraitQuery,
    ) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Trait(TraitPredicate {
                    trait_name: trait_name.into(),
                    query: Some(query),
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_trait_query<T: NamedMessage>(query: TraitQuery) -> QueryBuilder {
        Self::with_trait_name_query(T::full_name(), query)
    }

    pub fn with_entity_id<E: Into<String>>(id: E) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Ids(IdsPredicate {
                    ids: vec![id.into()],
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_entity_ids<I, E>(ids: I) -> QueryBuilder
    where
        I: Iterator<Item = E>,
        E: Into<String>,
    {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Ids(IdsPredicate {
                    ids: ids.map(|i| i.into()).collect(),
                })),
                ..Default::default()
            },
        }
    }

    pub fn all() -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::All(AllPredicate {})),
                ..Default::default()
            },
        }
    }

    pub fn failed() -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Test(TestPredicate {
                    success: false,
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_paging(mut self, paging: Paging) -> Self {
        self.query.paging = Some(paging);
        self
    }

    pub fn with_count(mut self, count: u32) -> Self {
        match self.query.paging.as_mut() {
            Some(paging) => paging.count = count,
            None => {
                self.query.paging = Some(Paging {
                    count,
                    ..Default::default()
                })
            }
        }

        self
    }

    pub fn with_projection<P: Into<ProjectionWrapper>>(mut self, projection: P) -> Self {
        self.query.projections.push(projection.into().0);
        self
    }

    pub fn with_projections<P: Into<ProjectionWrapper>>(mut self, projections: Vec<P>) -> Self {
        self.query.projections = projections.into_iter().map(|p| p.into().0).collect();
        self
    }

    pub fn skip_if_results_equals(mut self, result_hash: ResultHash) -> Self {
        self.query.result_hash = result_hash;
        self
    }

    pub fn paging_or_default(&self) -> Paging {
        self.query.paging.clone().unwrap_or_else(default_paging)
    }

    pub fn with_watch_token(mut self, token: WatchToken) -> Self {
        self.query.watch_token = token;
        self
    }

    pub fn order_by_field<F: Into<String>>(mut self, field: F, ascending: bool) -> Self {
        if self.query.ordering.is_none() {
            self.query.ordering = Some(Ordering::default());
        }

        if let Some(ordering) = self.query.ordering.as_mut() {
            ordering.value = Some(ordering::Value::Field(field.into()));
            ordering.ascending = ascending;
        }

        self
    }

    pub fn order_by_operations(mut self, ascending: bool) -> Self {
        if self.query.ordering.is_none() {
            self.query.ordering = Some(Ordering::default());
        }

        if let Some(ordering) = self.query.ordering.as_mut() {
            ordering.value = Some(ordering::Value::OperationId(true));
            ordering.ascending = ascending;
        }

        self
    }

    pub fn order_ascending(mut self, ascending: bool) -> Self {
        if self.query.ordering.is_none() {
            self.query.ordering = Some(Ordering::default());
        }

        if let Some(ordering) = self.query.ordering.as_mut() {
            ordering.ascending = ascending;
        }

        self
    }

    pub fn include_deleted(mut self) -> Self {
        self.query.include_deleted = true;
        self
    }

    pub fn build(self) -> EntityQuery {
        self.query
    }
}

pub struct TraitQueryBuilder {
    query: TraitQuery,
}

impl TraitQueryBuilder {
    pub fn matches<S: Into<String>>(query: S) -> TraitQueryBuilder {
        TraitQueryBuilder {
            query: TraitQuery {
                predicate: Some(trait_query::Predicate::Match(MatchPredicate {
                    query: query.into(),
                })),
            },
        }
    }

    pub fn field_equals<F: Into<String>, V: Into<FieldPredicateValueWrapper>>(
        field: F,
        value: V,
    ) -> TraitQueryBuilder {
        TraitQueryBuilder {
            query: TraitQuery {
                predicate: Some(trait_query::Predicate::Field(TraitFieldPredicate {
                    field: field.into(),
                    value: Some(value.into().0),
                    operator: trait_field_predicate::Operator::Equal.into(),
                })),
            },
        }
    }

    pub fn field_references<F: Into<String>, V: Into<ReferencePredicateWrapper>>(
        field: F,
        reference: V,
    ) -> TraitQueryBuilder {
        TraitQueryBuilder {
            query: TraitQuery {
                predicate: Some(trait_query::Predicate::Reference(
                    TraitFieldReferencePredicate {
                        field: field.into(),
                        reference: Some(reference.into().0),
                    },
                )),
            },
        }
    }

    pub fn build(self) -> TraitQuery {
        self.query
    }
}

pub struct ProjectionBuilder {
    projection: Projection,
}

impl ProjectionBuilder {
    pub fn for_package_prefix<S: Into<String>>(package: S) -> ProjectionBuilder {
        ProjectionBuilder {
            projection: Projection {
                package: vec![package.into()],
                ..Default::default()
            },
        }
    }

    pub fn for_trait_name<S: Into<String>>(trait_name: S) -> ProjectionBuilder {
        ProjectionBuilder {
            projection: Projection {
                package: vec![format!("{}$", trait_name.into())],
                ..Default::default()
            },
        }
    }

    pub fn for_trait<T: NamedMessage>() -> ProjectionBuilder {
        Self::for_trait_name(T::full_name())
    }

    pub fn for_all() -> ProjectionBuilder {
        ProjectionBuilder {
            projection: Default::default(),
        }
    }

    pub fn skip(mut self) -> ProjectionBuilder {
        self.projection.skip = true;
        self
    }

    pub fn return_fields(mut self, field_ids: Vec<FieldId>) -> ProjectionBuilder {
        self.projection.field_ids = field_ids;
        self
    }

    pub fn return_field_groups(mut self, field_groups: Vec<FieldId>) -> ProjectionBuilder {
        self.projection.field_group_ids = field_groups;
        self
    }

    pub fn build(self) -> Projection {
        self.projection
    }
}

pub struct FieldPredicateValueWrapper(trait_field_predicate::Value);

impl Into<FieldPredicateValueWrapper> for EntityId {
    fn into(self) -> FieldPredicateValueWrapper {
        FieldPredicateValueWrapper(trait_field_predicate::Value::String(self))
    }
}

impl Into<FieldPredicateValueWrapper> for &str {
    fn into(self) -> FieldPredicateValueWrapper {
        FieldPredicateValueWrapper(trait_field_predicate::Value::String(self.to_string()))
    }
}

pub struct ReferencePredicateWrapper(ReferencePredicate);

impl Into<ReferencePredicateWrapper> for EntityId {
    fn into(self) -> ReferencePredicateWrapper {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: self,
            trait_id: String::new(),
        })
    }
}

impl Into<ReferencePredicateWrapper> for (EntityId, TraitId) {
    fn into(self) -> ReferencePredicateWrapper {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: self.0,
            trait_id: self.1,
        })
    }
}

impl Into<ReferencePredicateWrapper> for &str {
    fn into(self) -> ReferencePredicateWrapper {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: self.to_string(),
            trait_id: String::new(),
        })
    }
}

impl Into<ReferencePredicateWrapper> for (&str, &str) {
    fn into(self) -> ReferencePredicateWrapper {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: self.0.to_string(),
            trait_id: self.1.to_string(),
        })
    }
}

pub struct ProjectionWrapper(Projection);

impl Into<ProjectionWrapper> for Projection {
    fn into(self) -> ProjectionWrapper {
        ProjectionWrapper(self)
    }
}

impl Into<ProjectionWrapper> for ProjectionBuilder {
    fn into(self) -> ProjectionWrapper {
        ProjectionWrapper(self.build())
    }
}

pub fn default_paging() -> Paging {
    Paging {
        count: 10,
        ..Default::default()
    }
}

pub fn validate_paging(paging: &mut Paging) {
    if paging.count == 0 {
        paging.count = 10;
    }
}

pub fn query_to_request_frame(
    query: &EntityQuery,
) -> Result<CapnpFrameBuilder<query_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<query_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = query.encode_to_vec()?;
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

pub fn query_from_request_frame<I>(
    frame: TypedCapnpFrame<I, query_request::Owned>,
) -> Result<EntityQuery, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;

    let query = EntityQuery::decode(data)?;

    Ok(query)
}

pub fn watched_query_to_request_frame(
    query: &EntityQuery,
) -> Result<CapnpFrameBuilder<watched_query_request::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<watched_query_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    let buf = query.encode_to_vec()?;
    msg_builder.set_request(&buf);

    Ok(frame_builder)
}

pub fn watched_query_from_request_frame<I>(
    frame: TypedCapnpFrame<I, watched_query_request::Owned>,
) -> Result<EntityQuery, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    let data = reader.get_request()?;

    let query = EntityQuery::decode(data)?;

    Ok(query)
}

pub fn query_results_to_response_frame(
    result: Result<EntityResults, Error>,
) -> Result<CapnpFrameBuilder<query_response::Owned>, Error> {
    let mut frame_builder = CapnpFrameBuilder::<query_response::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();

    match result {
        Ok(res) => {
            let buf = res.encode_to_vec()?;
            msg_builder.set_response(&buf);
        }
        Err(err) => {
            msg_builder.set_error(&err.to_string());
        }
    }

    Ok(frame_builder)
}

pub fn query_results_from_response_frame<I>(
    frame: TypedCapnpFrame<I, query_response::Owned>,
) -> Result<EntityResults, Error>
where
    I: FrameReader,
{
    let reader = frame.get_reader()?;
    if reader.has_error() {
        Err(Error::Remote(reader.get_error()?.to_owned()))
    } else {
        let data = reader.get_response()?;
        let res = EntityResults::decode(data)?;
        Ok(res)
    }
}
