#[cfg(feature = "local_store")]
pub mod local;
pub mod remote;

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult, WatchedQuery};
use futures::{Future, Stream};

pub type AsyncResult<I> = Box<dyn Future<Item = I, Error = Error> + Send>;
pub type ResultStream<I> = Box<dyn Stream<Item = I, Error = Error> + Send>;

pub trait AsyncStore {
    fn mutate(&self, mutation: Mutation) -> AsyncResult<MutationResult>;
    fn query(&self, query: Query) -> AsyncResult<QueryResult>;
    fn watched_query(&self, watched_query: WatchedQuery) -> ResultStream<QueryResult>;
}
