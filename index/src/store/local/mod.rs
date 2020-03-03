mod entity_index;
mod mutation_index;
mod store;
mod top_results_iter;
mod watched_queries;

pub use entity_index::{EntityIndex, EntityIndexConfig};
pub use store::{Store, StoreHandle};

#[cfg(test)]
mod test_store;

#[cfg(test)]
pub use test_store::TestStore;
