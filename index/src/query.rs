use crate::domain::entity::EntityId;

#[serde(rename_all = "snake_case", tag = "type")]
#[derive(Serialize, Deserialize)]
pub enum Query {
    WithTrait(WithTraitQuery),
    Conjunction(ConjunctionQuery),
    Match(MatchQuery),
    IdEqual(EntityId),
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
        Query::IdEqual(entity_id.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() -> Result<(), failure::Error> {
        let query = Query::with_trait("trait");
        assert!(serde_json::to_string(&query).is_ok());
        Ok(())
    }

}
