use exocore_protos::{
    generated::exocore_store::{
        entity_query, ordering, trait_field_predicate, trait_query, EntityQuery, MatchPredicate,
        Ordering, Paging, ReferencePredicate, TraitFieldPredicate, TraitFieldReferencePredicate,
        TraitPredicate, TraitQuery,
    },
    message::NamedMessage,
    reflect::FieldId,
    store::{
        AllPredicate, IdsPredicate, OperationsPredicate, Projection, QueryStringPredicate,
        Reference,
    },
};

use crate::{
    entity::{EntityId, TraitId},
    mutation::OperationId,
};

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
                    ..Default::default()
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

    pub fn with_id<E: Into<String>>(id: E) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Ids(IdsPredicate {
                    ids: vec![id.into()],
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_ids<I>(ids: I) -> QueryBuilder
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Ids(IdsPredicate {
                    ids: ids.into_iter().map(|i| i.into()).collect(),
                })),
                ..Default::default()
            },
        }
    }

    pub fn from_query_string<T: Into<String>>(query: T) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::QueryString(QueryStringPredicate {
                    query: query.into(),
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

    #[cfg(any(test, feature = "tests-utils"))]
    pub fn test(success: bool) -> QueryBuilder {
        use exocore_protos::generated::exocore_store::TestPredicate;
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Test(TestPredicate { success })),
                ..Default::default()
            },
        }
    }

    pub fn with_paging(mut self, paging: Paging) -> Self {
        self.query.paging = Some(paging);
        self
    }

    pub fn count(mut self, count: u32) -> Self {
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

    pub fn project<P: Into<ProjectionWrapper>>(mut self, projection: P) -> Self {
        self.query.projections.push(projection.into().0);
        self
    }

    pub fn projects<I>(mut self, projections: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ProjectionWrapper>,
    {
        self.query.projections = projections.into_iter().map(|p| p.into().0).collect();
        self
    }

    pub fn skip_if_results_equals(mut self, result_hash: ResultHash) -> Self {
        self.query.result_hash = result_hash;
        self
    }

    pub fn paging_or_default(&self) -> Paging {
        self.query.paging.unwrap_or_else(default_paging)
    }

    pub fn with_watch_token(mut self, token: WatchToken) -> Self {
        self.query.watch_token = token;
        self
    }

    pub fn order_by_field<F: Into<String>>(self, field: F, ascending: bool) -> Self {
        self.mapped_ordering(|ordering| {
            ordering.value = Some(ordering::Value::Field(field.into()));
            ordering.ascending = ascending;
        })
    }

    pub fn order_by_operations(self, ascending: bool) -> Self {
        self.mapped_ordering(|ordering| {
            ordering.value = Some(ordering::Value::OperationId(true));
            ordering.ascending = ascending;
        })
    }

    pub fn order_by_score(
        self,
        ascending: bool,
        recency_boost: bool,
        reference_boost: bool,
    ) -> Self {
        self.mapped_ordering(|ordering| {
            ordering.value = Some(ordering::Value::Score(true));
            ordering.ascending = ascending;
            ordering.no_recency_boost = !recency_boost;
            ordering.no_reference_boost = !reference_boost;
        })
    }

    pub fn order_ascending(self, ascending: bool) -> Self {
        self.mapped_ordering(|ordering| ordering.ascending = ascending)
    }

    pub fn mapped_ordering<F: FnOnce(&mut Ordering)>(mut self, f: F) -> Self {
        match self.query.ordering.as_mut() {
            Some(ordering) => f(ordering),
            None => {
                let mut ordering = Ordering::default();
                f(&mut ordering);
                self.query.ordering = Some(ordering);
            }
        }

        self
    }

    pub fn include_deleted(mut self) -> Self {
        self.query.include_deleted = true;
        self
    }

    pub fn programmatic(mut self) -> Self {
        self.query.programmatic = true;
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
                    ..Default::default()
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

    pub fn from_query_string<S: Into<String>>(query: S) -> TraitQueryBuilder {
        TraitQueryBuilder {
            query: TraitQuery {
                predicate: Some(trait_query::Predicate::QueryString(QueryStringPredicate {
                    query: query.into(),
                })),
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

    pub fn return_all(self) -> ProjectionBuilder {
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

impl From<EntityId> for FieldPredicateValueWrapper {
    fn from(id: EntityId) -> Self {
        FieldPredicateValueWrapper(trait_field_predicate::Value::String(id))
    }
}

impl From<&str> for FieldPredicateValueWrapper {
    fn from(s: &str) -> Self {
        FieldPredicateValueWrapper(trait_field_predicate::Value::String(s.to_string()))
    }
}

pub struct ReferencePredicateWrapper(ReferencePredicate);

impl From<EntityId> for ReferencePredicateWrapper {
    fn from(id: EntityId) -> Self {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: id,
            trait_id: String::new(),
        })
    }
}

impl From<(EntityId, TraitId)> for ReferencePredicateWrapper {
    fn from(tup: (EntityId, TraitId)) -> Self {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: tup.0,
            trait_id: tup.1,
        })
    }
}

impl From<&str> for ReferencePredicateWrapper {
    fn from(s: &str) -> Self {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: s.to_string(),
            trait_id: String::new(),
        })
    }
}

impl From<(&str, &str)> for ReferencePredicateWrapper {
    fn from(tup: (&str, &str)) -> Self {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: tup.0.to_string(),
            trait_id: tup.1.to_string(),
        })
    }
}

impl From<Reference> for ReferencePredicateWrapper {
    fn from(r: Reference) -> Self {
        ReferencePredicateWrapper(ReferencePredicate {
            entity_id: r.entity_id,
            trait_id: r.trait_id,
        })
    }
}

pub struct ProjectionWrapper(Projection);

impl From<Projection> for ProjectionWrapper {
    fn from(p: Projection) -> Self {
        ProjectionWrapper(p)
    }
}

impl From<ProjectionBuilder> for ProjectionWrapper {
    fn from(b: ProjectionBuilder) -> Self {
        ProjectionWrapper(b.build())
    }
}

pub fn default_paging() -> Paging {
    Paging {
        count: 10,
        ..Default::default()
    }
}

pub fn fill_default_paging(paging: &mut Paging) {
    if paging.count == 0 {
        paging.count = 10;
    }
}
