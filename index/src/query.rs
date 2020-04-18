use prost::Message;

use exocore_core::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_core::protos::generated::exocore_index::{
    entity_query, EntityQuery, EntityResults, IdPredicate, MatchPredicate, Paging, Sorting,
    TestPredicate, TraitPredicate,
};
use exocore_core::protos::generated::index_transport_capnp::watched_query_request;
use exocore_core::protos::generated::index_transport_capnp::{query_request, query_response};
use exocore_core::protos::prost::ProstMessageExt;

use crate::error::Error;

pub type WatchToken = u64;
pub type ResultHash = u64;

#[derive(Clone)]
pub struct QueryBuilder {
    query: EntityQuery,
}

impl QueryBuilder {
    pub fn match_text<S: Into<String>>(query: S) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Match(MatchPredicate {
                    query: query.into(),
                })),
                ..Default::default()
            },
        }
    }

    pub fn with_trait<S: Into<String>>(trait_name: S) -> QueryBuilder {
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

    pub fn with_entity_id<S: Into<String>>(entity_id: S) -> QueryBuilder {
        QueryBuilder {
            query: EntityQuery {
                predicate: Some(entity_query::Predicate::Id(IdPredicate {
                    id: entity_id.into(),
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

    pub fn order(mut self, ascending: bool) -> Self {
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

pub fn default_paging() -> Paging {
    Paging {
        count: 10,
        ..Default::default()
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
