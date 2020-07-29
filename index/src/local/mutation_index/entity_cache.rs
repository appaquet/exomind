use super::EntityMutationResults;
use crate::entity::EntityId;
use lru::LruCache;
use std::sync::Mutex;

// TODO: Fix useless cloning of &str to &String once https://github.com/jeromefroe/lru-rs/issues/85 is resolved

/// Cache of entity mutations results when fetched using the `fetch_entity_mutations`
/// method on the `MutationIndex`.
///
/// The `fetch_entity_mutation` method is use massively when iterating through results
/// in the `EntityIndex` to know if an entity's matched mutation is still active, and
/// get the chain's storage information if the result is to be returned.
pub struct EntityMutationsCache {
    cache: Mutex<LruCache<EntityId, EntityMutationResults>>,
}

impl EntityMutationsCache {
    pub fn new(size: usize) -> EntityMutationsCache {
        EntityMutationsCache {
            cache: Mutex::new(LruCache::new(size)),
        }
    }

    pub fn get(&self, entity_id: &str) -> Option<EntityMutationResults> {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        let entity_id = entity_id.to_string();
        cache.get(&entity_id).cloned()
    }

    pub fn put(&self, entity_id: &str, results: EntityMutationResults) {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        let entity_id = entity_id.to_string();
        cache.put(entity_id, results);
    }

    pub fn remove(&self, entity_id: &str) {
        let mut cache = self
            .cache
            .lock()
            .expect("Entity mutations cache lock is poisoned");

        let entity_id = entity_id.to_string();
        cache.pop(&entity_id);
    }
}
