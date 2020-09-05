use super::MutationIndex;
use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use crate::ordering::OrderingValueWrapper;
use chrono::{DateTime, Utc};
use exocore_chain::block::BlockOffset;
use exocore_chain::operation::OperationId;
use exocore_core::protos::generated::exocore_index::{EntityQuery, Paging};
use std::borrow::Borrow;

/// Iterates through all results matching a given initial query using the
/// next_page score when a page got emptied.
pub struct MutationResultsIterator<'i, Q: Borrow<EntityQuery>> {
    pub index: &'i MutationIndex,
    pub query: Q,
    pub total_results: usize,
    pub current_results: std::vec::IntoIter<MutationMetadata>,
    pub next_page: Option<Paging>,
    pub max_pages: usize,
}

impl<'i, Q: Borrow<EntityQuery>> Iterator for MutationResultsIterator<'i, Q> {
    type Item = MutationMetadata;

    fn next(&mut self) -> Option<Self::Item> {
        let next_result = self.current_results.next();
        if let Some(next_result) = next_result {
            Some(next_result)
        } else {
            let mut query = self.query.borrow().clone();
            query.paging = Some(self.next_page.clone()?);

            if self.max_pages == 0 {
                warn!(
                    "Too many page fetched. Stopping here. Last query={:?}",
                    query
                );
                return None;
            }

            let results = self
                .index
                .search(&query)
                .expect("Couldn't get another page from initial iterator query");
            self.next_page = results.next_page;
            self.current_results = results.mutations.into_iter();
            self.max_pages -= 1;

            self.current_results.next()
        }
    }
}

/// Collection of `MutationMetadata`
pub struct MutationResults {
    pub mutations: Vec<MutationMetadata>,
    pub total: usize,
    pub remaining: usize,
    pub next_page: Option<Paging>,
}

#[derive(Clone)]
pub struct EntityMutationResults {
    pub mutations: Vec<MutationMetadata>,
}

/// Indexed trait / entity mutation metadata returned as a result of a query.
#[derive(Debug, Clone)]
pub struct MutationMetadata {
    pub operation_id: OperationId,
    pub block_offset: Option<BlockOffset>,
    pub entity_id: EntityId,
    pub mutation_type: MutationType,
    pub sort_value: OrderingValueWrapper,
}

#[derive(Debug, Clone)]
pub enum MutationType {
    TraitPut(PutTraitMetadata),
    TraitTombstone(TraitId),
    EntityTombstone,
}

#[derive(Debug, Clone)]
pub struct PutTraitMetadata {
    pub trait_id: TraitId,
    pub trait_type: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
}

impl MutationType {
    pub const TRAIT_TOMBSTONE_ID: u64 = 0;
    pub const TRAIT_PUT_ID: u64 = 1;
    pub const ENTITY_TOMBSTONE_ID: u64 = 2;

    pub fn new(
        document_type_id: u64,
        opt_trait_id: Option<TraitId>,
    ) -> Result<MutationType, Error> {
        match document_type_id {
            Self::TRAIT_TOMBSTONE_ID => Ok(MutationType::TraitTombstone(opt_trait_id.unwrap())),
            Self::TRAIT_PUT_ID => Ok(MutationType::TraitPut(PutTraitMetadata {
                trait_id: opt_trait_id.unwrap(),
                trait_type: None,
                creation_date: None,
                modification_date: None,
            })),
            Self::ENTITY_TOMBSTONE_ID => Ok(MutationType::EntityTombstone),
            _ => Err(Error::Fatal(format!(
                "Invalid document type id {}",
                document_type_id
            ))),
        }
    }
}
