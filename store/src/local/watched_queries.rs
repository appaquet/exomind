use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use exocore_protos::generated::exocore_store::{EntityQuery, EntityResults};
use futures::channel::mpsc;

use crate::{
    error::Error,
    query::{ResultHash, WatchToken},
};

/// Collection of queries that have been requested to be watched and that the
/// user wants to be notified when their results have changed. The actual
/// management of the queries is done by the store.
pub struct WatchedQueries {
    inner: Mutex<Inner>,
}

impl WatchedQueries {
    pub fn new() -> WatchedQueries {
        WatchedQueries {
            inner: Mutex::new(Inner {
                queries: HashMap::new(),
            }),
        }
    }

    pub fn track_query(
        &self,
        token: WatchToken,
        query: &EntityQuery,
        sender: Arc<Mutex<mpsc::Sender<Result<EntityResults, Error>>>>,
    ) {
        let mut inner = self.inner.lock().expect("Inner got poisoned");

        let watched_query = RegisteredWatchedQuery {
            token,
            sender,
            query: Box::new(query.clone()),
            last_hash: None,
        };

        inner.queries.insert(token, watched_query);
    }

    pub fn update_query_results(&self, token: WatchToken, results: &EntityResults) -> bool {
        let mut inner = self.inner.lock().expect("Inner got poisoned");

        if let Some(current_watched) = inner.queries.get_mut(&token) {
            let should_reply = current_watched.last_hash != Some(results.hash);
            current_watched.last_hash = Some(results.hash);
            current_watched.query.result_hash = results.hash;

            should_reply
        } else {
            false
        }
    }

    pub fn unwatch_query(&self, token: WatchToken) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.queries.remove(&token);
            debug!(
                "Dropped watched query {}. {} watched queries left.",
                token,
                inner.queries.len()
            );
        }
    }

    pub fn queries(&self) -> Vec<RegisteredWatchedQuery> {
        let inner = self.inner.lock().expect("Inner got poisoned");
        inner.queries.values().cloned().collect()
    }

    #[cfg(test)]
    pub fn clear(&self) {
        let mut inner = self.inner.lock().expect("Inner got poisoned");
        inner.queries.clear();
    }
}

struct Inner {
    queries: HashMap<WatchToken, RegisteredWatchedQuery>,
}

#[derive(Clone)]
pub struct RegisteredWatchedQuery {
    pub(crate) token: WatchToken,
    pub(crate) sender: Arc<Mutex<mpsc::Sender<Result<EntityResults, Error>>>>,
    pub(crate) query: Box<EntityQuery>,
    pub(crate) last_hash: Option<ResultHash>,
}
