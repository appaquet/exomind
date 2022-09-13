use std::{num::NonZeroUsize, sync::Mutex};

use lru::LruCache;

use super::EntityMutationResults;
use crate::entity::EntityId;

/// Cache of entity mutations results when fetched using the
/// `fetch_entity_mutations` method on the `MutationIndex`.
///
/// The `fetch_entity_mutation` method is use massively when iterating through
/// results in the `EntityIndex` to know if an entity's matched mutation is
/// still active, and get the chain's storage information if the result is to be
/// returned.
pub struct EntityMutationsCache {
    cache: Mutex<LruCache<EntityId, EntityMutationResults>>,
}

impl EntityMutationsCache {
    pub fn new(size: NonZeroUsize) -> EntityMutationsCache {
        EntityMutationsCache {
            cache: Mutex::new(LruCache::new(size)),
        }
    }

    pub fn get(&self, entity_id: &str) -> Option<EntityMutationResults> {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        cache.get(entity_id).cloned()
    }

    pub fn put(&self, entity_id: &str, results: EntityMutationResults) {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        cache.put(entity_id.to_string(), results);
    }

    pub fn remove(&self, entity_id: &str) {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        cache.pop(entity_id);
    }
}
