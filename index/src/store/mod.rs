#[cfg(feature = "local_store")]
pub mod local;
pub mod remote;

use crate::error::Error;
use crate::mutation::{Mutation, MutationResult};
use crate::query::{Query, QueryResult};
use futures::Future;

pub type AsyncResult<I> = Box<dyn Future<Item = I, Error = Error> + Send>;

pub trait AsyncStore {
    fn mutate(&self, mutation: Mutation) -> AsyncResult<MutationResult>;
    fn query(&self, query: Query) -> AsyncResult<QueryResult>;
}
