use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use exocore_protos::{
    apps::{out_message::OutMessageType, InMessage, MessageStatus, OutMessage},
    generated::store::{EntityQuery, EntityResults},
    prost::Message,
    store::MutationResult,
};
use exocore_store::mutation::MutationRequestLike;
use futures::channel::oneshot;

use crate::{
    prelude::{sleep, spawn},
    time::{now, Timestamp},
};

// Keep in sync with remote store `ClientConfiguration`
const MUTATION_TIMEOUT: Duration = Duration::from_secs(5);
const QUERY_TIMEOUT: Duration = Duration::from_secs(10);
const TIMEOUT_CHECK_INTERVAL: Duration = Duration::from_secs(1);

/// Exocore entities store client.
pub struct Store {
    next_rdv: AtomicUsize,
    inner: Mutex<Inner>,

    #[cfg(test)]
    host_message_sender: Option<Box<dyn Fn(OutMessage) -> MessageStatus + Send + Sync>>,
}

#[derive(Default)]
struct Inner {
    pending_mutations: HashMap<usize, OneshotRequest<MutationResult>>,
    pending_queries: HashMap<usize, OneshotRequest<EntityResults>>,
}

struct OneshotRequest<T> {
    sender: oneshot::Sender<Result<T, StoreError>>,
    timeout: Timestamp,
}

impl Store {
    pub(crate) fn new() -> Store {
        Store {
            next_rdv: AtomicUsize::new(0),
            inner: Mutex::new(Inner::default()),

            #[cfg(test)]
            host_message_sender: None,
        }
    }

    pub async fn mutate(
        self: &Arc<Store>,
        mutation: impl Into<MutationRequestLike>,
    ) -> Result<MutationResult, StoreError> {
        let mutation = mutation.into();

        let rdv = self.next_rdv.fetch_add(1, Ordering::SeqCst);
        let msg_type = OutMessageType::StoreMutationRequest;
        let msg = OutMessage {
            r#type: msg_type.into(),
            rendez_vous_id: rdv as u32,
            data: mutation.encode_to_vec(),
        };

        let (sender, receiver) = oneshot::channel();
        {
            let mut inner = self.inner.lock().unwrap();
            let pending = OneshotRequest {
                sender,
                timeout: now() + QUERY_TIMEOUT,
            };
            inner.pending_mutations.insert(rdv, pending);
        }

        self.send_host_message(msg)?;

        receiver.await.map_err(StoreError::from)?
    }

    pub async fn query(self: &Arc<Store>, query: EntityQuery) -> Result<EntityResults, StoreError> {
        let rdv = self.next_rdv.fetch_add(1, Ordering::SeqCst);
        let msg_type = OutMessageType::StoreEntityQuery;
        let msg = OutMessage {
            r#type: msg_type.into(),
            rendez_vous_id: rdv as u32,
            data: query.encode_to_vec(),
        };

        let (sender, receiver) = oneshot::channel();
        {
            let mut inner = self.inner.lock().unwrap();
            let pending = OneshotRequest {
                sender,
                timeout: now() + MUTATION_TIMEOUT,
            };
            inner.pending_queries.insert(rdv, pending);
        }

        self.send_host_message(msg)?;

        receiver.await.map_err(StoreError::from)?
    }

    pub(crate) fn handle_mutation_result(&self, msg: InMessage) -> Result<(), MessageStatus> {
        let mut inner = self.inner.lock().unwrap();
        let rdv = msg.rendez_vous_id as usize;

        if let Some(req) = inner.pending_mutations.remove(&rdv) {
            let results = if msg.error.is_empty() {
                Ok(MutationResult::decode(msg.data.as_ref()).map_err(|err| {
                    error!("Error decoding incoming mutation result: {}", err);
                    MessageStatus::DecodeError
                })?)
            } else {
                Err(StoreError::Remote(msg.error))
            };
            let _ = req.sender.send(results);
        }

        Ok(())
    }

    pub(crate) fn handle_query_results(&self, msg: InMessage) -> Result<(), MessageStatus> {
        let mut inner = self.inner.lock().unwrap();
        let rdv = msg.rendez_vous_id as usize;

        if let Some(req) = inner.pending_queries.remove(&rdv) {
            let results = if msg.error.is_empty() {
                Ok(EntityResults::decode(msg.data.as_ref()).map_err(|err| {
                    error!("Error decoding incoming query results: {}", err);
                    MessageStatus::DecodeError
                })?)
            } else {
                Err(StoreError::Remote(msg.error))
            };
            let _ = req.sender.send(results);
        }

        Ok(())
    }

    pub(crate) fn start(self: &Arc<Store>) {
        let store = self.clone();
        spawn(async move {
            loop {
                let now = now();

                {
                    let mut inner = store.inner.lock().unwrap();
                    check_timed_out_queries(&mut inner, now);
                    check_timed_out_mutations(&mut inner, now);
                }

                sleep(TIMEOUT_CHECK_INTERVAL).await;
            }
        });
    }

    #[cfg(not(test))]
    fn send_host_message(&self, msg: OutMessage) -> Result<(), StoreError> {
        let encoded = msg.encode_to_vec();
        unsafe {
            let code = crate::binding::__exocore_host_out_message(encoded.as_ptr(), encoded.len());
            StoreError::from_message_status(code as i32)?;
        }

        Ok(())
    }

    #[cfg(test)]
    fn send_host_message(&self, msg: OutMessage) -> Result<(), StoreError> {
        let sender = self.host_message_sender.as_ref().unwrap();
        let code = sender(msg);
        StoreError::from_message_status(code as i32)?;

        Ok(())
    }
}

fn check_timed_out_queries(inner: &mut std::sync::MutexGuard<Inner>, now: Timestamp) {
    let mut timed_out = Vec::new();
    for (rdv, query) in &inner.pending_queries {
        if query.timeout < now {
            timed_out.push(*rdv);
        }
    }

    for rdv in timed_out {
        inner.pending_queries.remove(&rdv);
    }
}

fn check_timed_out_mutations(inner: &mut std::sync::MutexGuard<Inner>, now: Timestamp) {
    let mut timed_out = Vec::new();
    for (rdv, query) in &inner.pending_mutations {
        if query.timeout < now {
            timed_out.push(*rdv);
        }
    }

    for rdv in timed_out {
        inner.pending_mutations.remove(&rdv);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    #[error("Host message error: {0:?}")]
    HostMessage(MessageStatus),
    #[error("Remote store error: {0:?}")]
    Remote(String),
    #[error("Query or mutation got cancelled or timed out")]
    Cancelled(#[from] oneshot::Canceled),
}

impl StoreError {
    fn from_message_status(code: i32) -> Result<(), StoreError> {
        match MessageStatus::try_from(code) {
            Ok(MessageStatus::Ok) => Ok(()),
            Ok(status) => Err(StoreError::HostMessage(status)),
            Err(err) => Err(StoreError::Unknown(anyhow::anyhow!(
                "Unknown message status code: {}. err: {err}",
                code
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use exocore_protos::{apps::in_message::InMessageType, store::MutationRequest};
    use futures::{channel::mpsc, StreamExt};

    use super::*;

    #[tokio::test]
    async fn test_mutation() {
        let (mut out_msg_rcv, store) = create_test_store();

        // spawn a mutation request
        let (res_sender, mut res_receiver) = oneshot::channel();
        {
            let store = store.clone();
            tokio::spawn(async move {
                let res = store.mutate(MutationRequest::default()).await;
                res_sender.send(res).unwrap();
            });
        }

        // the mutation should have been sent to host
        let out_msg = out_msg_rcv.next().await.expect("no message sent to host");

        // mutation shouldn't have resolved yet since we didn't send results back
        assert!(res_receiver.try_recv().unwrap().is_none());

        // host sends back results
        store
            .handle_mutation_result(InMessage {
                r#type: InMessageType::StoreMutationResult.into(),
                data: MutationResult {
                    operation_ids: vec![123],
                    ..Default::default()
                }
                .encode_to_vec(),
                rendez_vous_id: out_msg.rendez_vous_id,
                error: String::new(),
            })
            .unwrap();

        // mutation should now have been resolved
        let res = res_receiver.await.unwrap().unwrap();
        assert_eq!(res.operation_ids, vec![123]);
    }

    #[tokio::test]
    async fn test_query() {
        let (mut out_msg_rcv, store) = create_test_store();

        // spawn a query
        let (res_sender, mut res_receiver) = oneshot::channel();
        {
            let store = store.clone();
            tokio::spawn(async move {
                let res = store.query(EntityQuery::default()).await;
                res_sender.send(res).unwrap();
            });
        }

        // the query should have been sent to host
        let out_msg = out_msg_rcv.next().await.expect("no message sent to host");

        // query shouldn't have resolved yet since we didn't send results back
        assert!(res_receiver.try_recv().unwrap().is_none());

        // host sends back results
        store
            .handle_query_results(InMessage {
                r#type: InMessageType::StoreEntityResults.into(),
                data: EntityResults {
                    estimated_count: 123,
                    ..Default::default()
                }
                .encode_to_vec(),
                rendez_vous_id: out_msg.rendez_vous_id,
                error: String::new(),
            })
            .unwrap();

        // query should now have been resolved
        let res = res_receiver.await.unwrap().unwrap();
        assert_eq!(res.estimated_count, 123);
    }

    fn create_test_store() -> (mpsc::Receiver<OutMessage>, Arc<Store>) {
        let (out_msg_sender, out_msg_rcv) = mpsc::channel(1);
        let store = {
            let mut store = Store::new();
            let out_msg_sender = Arc::new(Mutex::new(out_msg_sender));
            store.host_message_sender = Some(Box::new(move |msg| {
                let mut out_msg_sender = out_msg_sender.lock().unwrap();
                out_msg_sender.try_send(msg).unwrap();
                MessageStatus::Ok
            }));
            Arc::new(store)
        };

        (out_msg_rcv, store)
    }
}
