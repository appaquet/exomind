mod entities_index;
mod store;
mod top_results_iter;
mod traits_index;
mod watched_queries;

pub use entities_index::{EntitiesIndex, EntitiesIndexConfig};
pub use store::{Store, StoreHandle};

#[cfg(test)]
mod test_store;

#[cfg(test)]
pub use test_store::TestStore;
