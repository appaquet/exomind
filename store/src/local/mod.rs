mod entity_index;
mod mutation_index;
mod mutation_tracker;
mod store;
mod top_results;
mod watched_queries;
mod config;

#[cfg(feature = "local")]
pub use entity_index::{EntityIndex, EntityIndexConfig};
#[cfg(feature = "local")]
pub use store::{Store, StoreHandle};
#[cfg(feature = "local")]
pub use config::StoreConfig;

#[cfg(test)]
mod test_store;

#[cfg(test)]
pub use test_store::TestStore;
