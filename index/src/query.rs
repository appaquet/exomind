use crate::domain::entity::{Entity, EntityId};
use crate::domain::schema::Schema;
use crate::domain::serialization::with_schema;
use crate::error::Error;
use exocore_common::framing::{CapnpFrameBuilder, FrameReader, TypedCapnpFrame};
use exocore_common::protos::index_transport_capnp::{query_request, query_response};
use exocore_common::time::ConsistentTimestamp;
use std::sync::Arc;

pub type QueryId = ConsistentTimestamp;

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Serialize, Deserialize)]
pub enum Query {
    WithTrait(WithTraitQuery),
    Conjunction(ConjunctionQuery),
    Match(MatchQuery),
    IdEqual(IdEqualQuery),

    #[cfg(test)]
    TestFail(TestFailQuery),
}

impl Query {
    pub fn match_text<S: Into<String>>(query: S) -> Query {
        Query::Match(MatchQuery {
            query: query.into(),
        })
    }

    pub fn with_trait<S: Into<String>>(trait_name: S) -> Query {
        Query::WithTrait(WithTraitQuery {
            trait_name: trait_name.into(),
            trait_query: None,
        })
    }

    pub fn with_entity_id<S: Into<String>>(entity_id: S) -> Query {
        Query::IdEqual(IdEqualQuery {
            entity_id: entity_id.into(),
        })
    }

    #[cfg(test)]
    pub fn test_fail() -> Query {
        Query::TestFail(TestFailQuery {})
    }

    pub fn to_query_request_frame(
        &self,
        schema: &Arc<Schema>,
    ) -> Result<CapnpFrameBuilder<query_request::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::<query_request::Owned>::new();
        let mut msg_builder = frame_builder.get_builder();
        let serialized_query = with_schema(schema, || serde_json::to_vec(&self))?;
        msg_builder.set_request(&serialized_query);

        Ok(frame_builder)
    }

    pub fn from_query_request_frame<I>(
        schema: &Arc<Schema>,
        frame: TypedCapnpFrame<I, query_request::Owned>,
    ) -> Result<Query, Error>
    where
        I: FrameReader,
    {
        let reader = frame.get_reader()?;
        let data = reader.get_request()?;
        let query = with_schema(schema, || serde_json::from_slice(data))?;

        Ok(query)
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize)]
pub struct WithTraitQuery {
    pub trait_name: String,
    pub trait_query: Option<Box<Query>>,
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize)]
pub struct ConjunctionQuery {
    pub queries: Vec<Query>,
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize)]
pub struct MatchQuery {
    pub query: String,
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize)]
pub struct IdEqualQuery {
    pub entity_id: EntityId,
}

#[cfg(test)]
#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize)]
pub struct TestFailQuery {}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct SortToken(pub String);

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryPaging {
    pub from_token: Option<SortToken>,
    pub to_token: Option<SortToken>,
    pub count: u32,
}

impl QueryPaging {
    pub fn empty() -> QueryPaging {
        QueryPaging {
            from_token: None,
            to_token: None,
            count: 0,
        }
    }
}

///
/// Result of the query executed on index
///
#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub results: Vec<EntityResult>,
    pub total_estimated: u32,
    pub current_page: QueryPaging,
    pub next_page: Option<QueryPaging>,
    // TODO: currentPage, nextPage, queryToken
}

impl QueryResult {
    pub fn empty() -> QueryResult {
        QueryResult {
            results: vec![],
            total_estimated: 0,
            current_page: QueryPaging::empty(),
            next_page: None,
        }
    }

    pub fn result_to_response_frame(
        schema: &Arc<Schema>,
        result: Result<QueryResult, Error>,
    ) -> Result<CapnpFrameBuilder<query_response::Owned>, Error> {
        let mut frame_builder = CapnpFrameBuilder::<query_response::Owned>::new();
        let mut msg_builder = frame_builder.get_builder();

        match result {
            Ok(res) => {
                let serialized = with_schema(schema, || serde_json::to_vec(&res))?;
                msg_builder.set_response(&serialized);
            }
            Err(err) => {
                msg_builder.set_error(&err.to_string());
            }
        }

        Ok(frame_builder)
    }

    pub fn from_query_frame<I>(
        schema: &Arc<Schema>,
        frame: TypedCapnpFrame<I, query_response::Owned>,
    ) -> Result<QueryResult, Error>
    where
        I: FrameReader,
    {
        let reader = frame.get_reader()?;
        if reader.has_error() {
            Err(Error::Remote(reader.get_error()?.to_owned()))
        } else {
            let data = reader.get_response()?;
            let query_result = with_schema(schema, || serde_json::from_slice(data))?;
            Ok(query_result)
        }
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct EntityResult {
    pub entity: Entity,
    pub source: EntityResultSource,
    // TODO: sortToken:
}

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum EntityResultSource {
    Pending,
    Chain,
}
