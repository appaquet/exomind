use prost::Message;

use exocore_chain::operation::OperationId;
use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::exocore_index::{
    entity_query, sorting, trait_field_predicate, trait_query, EntityQuery, EntityResults,
    MatchPredicate, Paging, ReferencePredicate, Sorting, TestPredicate, TraitFieldPredicate,
    TraitFieldReferencePredicate, TraitPredicate, TraitQuery,
};
use exocore_core::protos::generated::index_transport_capnp::watched_query_request;
use exocore_core::protos::generated::index_transport_capnp::{query_request, query_response};
use exocore_core::protos::{
    index::{IdsPredicate, OperationsPredicate},
    prost::{NamedMessage, ProstMessageExt},
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
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Trait(TraitPredicate {
                    trait_name: T::full_name().to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        }
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
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Trait(TraitPredicate {
                    trait_name: T::full_name().into(),
                    query: Some(query),
                })),
                ..Default::default()
            },
        }
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

    pub fn only_summary(mut self) -> Self {
        self.query.summary = true;
        self
    }

    pub fn only_summary_if_equals(mut self, result_hash: ResultHash) -> Self {
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

    pub fn sort_by_field<F: Into<String>>(mut self, field: F) -> Self {
        self.query.sorting = Some(Sorting {
            value: Some(sorting::Value::Field(field.into())),
            ..Default::default()
        });
        self
    }

    pub fn sort_ascending(mut self, ascending: bool) -> Self {
        if let Some(sorting) = self.query.sorting.as_mut() {
            sorting.ascending = ascending;
        } else {
            self.query.sorting = Some(Sorting {
                ascending,
                value: None,
            });
        }

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
