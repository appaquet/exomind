use crate::domain::entity::Entity;
use crate::query::QueryPaging;

#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug)]
pub struct EntitiesResults {
    pub results: Vec<EntityResult>,
    pub total_estimated: u32,
    pub current_page: QueryPaging,
    pub next_page: Option<QueryPaging>,
    // TODO: currentPage, nextPage, queryToken
}

impl EntitiesResults {
    pub fn empty() -> EntitiesResults {
        EntitiesResults {
            results: vec![],
            total_estimated: 0,
            current_page: QueryPaging::empty(),
            next_page: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::SortToken;

    #[test]
    fn test_serialization() -> Result<(), failure::Error> {
        let entity = Entity::new("1234".to_string());
        let results = EntitiesResults {
            results: vec![EntityResult {
                entity,
                source: EntityResultSource::Pending,
            }],
            total_estimated: 0,
            current_page: QueryPaging {
                from_token: Some(SortToken("token".to_string())),
                to_token: None,
                count: 10,
            },
            next_page: None,
        };
        assert!(serde_json::to_string(&results).is_ok());

        Ok(())
    }

}
