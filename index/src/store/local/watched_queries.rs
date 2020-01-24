use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::channel::mpsc;

use exocore_common::time::Instant;

use crate::error::Error;
use crate::query::{ResultHash, WatchToken};
use exocore_common::protos::generated::exocore_index::{EntityQuery, EntityResults};

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

    pub fn update_query_results(
        &self,
        token: WatchToken,
        query: &EntityQuery,
        results: &EntityResults,
        sender: Arc<Mutex<mpsc::Sender<Result<EntityResults, Error>>>>,
    ) -> bool {
        let mut inner = self.inner.lock().expect("Inner got poisoned");

        if let Some(mut current_watched) = inner.queries.remove(&token) {
            let should_reply = current_watched.last_hash != results.hash;

            current_watched.last_hash = results.hash;
            inner.queries.insert(token, current_watched);

            should_reply
        } else {
            let watched_query = RegisteredWatchedQuery {
                token,
                sender,
                query: Arc::new(query.clone()),
                last_register: Instant::now(),
                last_hash: results.hash,
            };

            inner.queries.insert(token, watched_query);
            true
        }
    }

    pub fn unwatch_query(&self, token: WatchToken) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.queries.remove(&token);
        }
    }

    pub fn queries(&self) -> Vec<RegisteredWatchedQuery> {
        let inner = self.inner.lock().expect("Inner got poisoned");
        inner.queries.values().cloned().collect()
    }
}

struct Inner {
    queries: HashMap<WatchToken, RegisteredWatchedQuery>,
}

#[derive(Clone)]
pub struct RegisteredWatchedQuery {
    pub(crate) token: WatchToken,
    pub(crate) sender: Arc<Mutex<mpsc::Sender<Result<EntityResults, Error>>>>,
    pub(crate) query: Arc<EntityQuery>,
    pub(crate) last_register: Instant,
    pub(crate) last_hash: ResultHash,
}
