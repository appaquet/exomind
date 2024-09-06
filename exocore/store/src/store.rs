use async_trait::async_trait;
use exocore_protos::store::{EntityQuery, EntityResults, MutationResult};
use futures::Stream;

use crate::{error::Error, mutation::MutationRequestLike};

/// Abstraction for an entity store. There are two main implementation at
/// the moment: the local store and the remote store. The local store is a
/// locally hosted store, while the remote is a store that is on a remote node.
#[async_trait]
pub trait Store: Clone + Send + 'static {
    type WatchedQueryStream: Stream<Item = Result<EntityResults, Error>>;

    async fn mutate<M: Into<MutationRequestLike> + Send>(
        &self,
        request: M,
    ) -> Result<MutationResult, Error>;

    async fn query(&self, query: EntityQuery) -> Result<EntityResults, Error>;

    fn watched_query(&self, query: EntityQuery) -> Result<Self::WatchedQueryStream, Error>;
}
