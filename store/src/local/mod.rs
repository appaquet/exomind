mod entity_index;
mod mutation_index;
mod mutation_tracker;
mod store;
mod top_results;
mod watched_queries;

#[cfg(feature = "local")]
pub use entity_index::{EntityIndex, EntityIndexConfig};
pub use store::{Store, StoreHandle};

#[cfg(test)]
mod test_store;

#[cfg(test)]
pub use test_store::TestStore;
