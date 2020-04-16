use super::{MutationIndex, QueryPaging};
use crate::entity::{EntityId, TraitId};
use crate::error::Error;
use chrono::{DateTime, Utc};
use exocore_chain::block::BlockOffset;
use exocore_chain::operation::OperationId;
use exocore_core::protos::generated::exocore_index::EntityQuery;

/// Iterates through all results matching a given initial query using the
/// next_page score when a page got emptied.
pub struct ResultsIterator<'i, 'q> {
    pub index: &'i MutationIndex,
    pub query: &'q EntityQuery,
    pub total_results: usize,
    pub current_results: std::vec::IntoIter<MutationMetadata>,
    pub next_page: Option<QueryPaging>,
}

impl<'i, 'q> Iterator for ResultsIterator<'i, 'q> {
    type Item = MutationMetadata;

    fn next(&mut self) -> Option<Self::Item> {
        let next_result = self.current_results.next();
        if let Some(next_result) = next_result {
            Some(next_result)
        } else {
            let next_page = self.next_page.clone()?;
            let results = self
                .index
                .search(self.query, Some(next_page))
                .expect("Couldn't get another page from initial iterator query");
            self.next_page = results.next_page;
            self.current_results = results.results.into_iter();

            self.current_results.next()
        }
    }
}

/// Collection of `MutationMetadata`
pub struct MutationResults {
    pub results: Vec<MutationMetadata>,
    pub total_results: usize,
    pub remaining_results: usize,
    pub next_page: Option<QueryPaging>,
}

/// Indexed trait / entity mutation metadata returned as a result of a query.
#[derive(Debug, Clone)]
pub struct MutationMetadata {
    pub operation_id: OperationId,
    pub block_offset: Option<BlockOffset>,
    pub entity_id: EntityId,
    pub score: u64,
    pub mutation_type: MutationMetadataType,
}

#[derive(Debug, Clone)]
pub enum MutationMetadataType {
    TraitPut(PutTraitMetadata),
    TraitTombstone(TraitId),
    EntityTombstone,
}

#[derive(Debug, Clone)]
pub struct PutTraitMetadata {
    pub trait_id: TraitId,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
}

impl MutationMetadataType {
    pub const TRAIT_TOMBSTONE_ID: u64 = 0;
    pub const TRAIT_PUT_ID: u64 = 1;
    pub const ENTITY_TOMBSTONE_ID: u64 = 2;

    pub fn new(
        document_type_id: u64,
        opt_trait_id: Option<TraitId>,
    ) -> Result<MutationMetadataType, Error> {
        match document_type_id {
            Self::TRAIT_TOMBSTONE_ID => {
                Ok(MutationMetadataType::TraitTombstone(opt_trait_id.unwrap()))
            }
            Self::TRAIT_PUT_ID => Ok(MutationMetadataType::TraitPut(PutTraitMetadata {
                trait_id: opt_trait_id.unwrap(),
                creation_date: None,
                modification_date: None,
            })),
            Self::ENTITY_TOMBSTONE_ID => Ok(MutationMetadataType::EntityTombstone),
            _ => Err(Error::Fatal(format!(
                "Invalid document type id {}",
                document_type_id
            ))),
        }
    }
}

/// Wraps a sortable value so that it can be easily reversed when required or ignored if it's outside
/// of the requested paging.
#[derive(Clone)]
pub struct ResultScore<O: PartialOrd + PartialEq> {
    pub score: O,
    pub reverse: bool,
    pub operation_id: u64,

    // means that this result should not be returned (probably because it's not withing paging)
    // and should match less than any other non-ignored result
    pub ignore: bool,
}

impl<O: PartialOrd + PartialEq> PartialOrd for ResultScore<O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // an ignored result should be always be less, unless they are both
        if self.ignore && other.ignore {
            return Some(std::cmp::Ordering::Equal);
        } else if self.ignore {
            return Some(std::cmp::Ordering::Less);
        } else if other.ignore {
            return Some(std::cmp::Ordering::Greater);
        }

        let cmp = self.score.partial_cmp(&other.score);

        // if both are equal, we tie break using operation id to make sort stable
        let cmp = if cmp == Some(std::cmp::Ordering::Equal) {
            Some(self.operation_id.cmp(&other.operation_id))
        } else {
            cmp
        };

        // reverse if needed
        if self.reverse {
            cmp.map(|o| o.reverse())
        } else {
            cmp
        }
    }
}

impl<O: PartialOrd + PartialEq> PartialEq for ResultScore<O> {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}
