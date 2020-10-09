use crate::OutMessage;
use exocore_core::futures::{block_on, delay_for};
use futures::{channel::oneshot, lock::Mutex, FutureExt};
use std::{
    collections::HashMap, sync::atomic::AtomicU64, sync::atomic::Ordering, sync::Arc, sync::Weak,
    time::Duration,
};

use super::config::HTTPTransportConfig;

pub type RequestID = u64;

/// Tracks incoming HTTP requests for which we are waiting a reply from a
/// service.
pub struct RequestTracker {
    requests: Mutex<HashMap<RequestID, oneshot::Sender<OutMessage>>>,
    next_id: AtomicU64,
    config: HTTPTransportConfig,
}

impl RequestTracker {
    pub fn new(config: HTTPTransportConfig) -> RequestTracker {
        RequestTracker {
            requests: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(0),
            config,
        }
    }

    /// Pushes a new request for which we'll expect a reply from a service.
    pub async fn push(self: Arc<Self>) -> TrackedRequest {
        let mut requests = self.requests.lock().await;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let (sender, receiver) = oneshot::channel();
        let request = TrackedRequest {
            id,
            requests: Arc::downgrade(&self),
            receiver: Some(receiver),
            receive_timeout: self.config.request_timeout,
        };

        requests.insert(request.id, sender);

        request
    }

    /// Handles a reply from a service to be sent back to a request.
    pub async fn reply(&self, request_id: RequestID, message: OutMessage) {
        let sender = {
            let mut requests = self.requests.lock().await;
            requests.remove(&request_id)
        };

        if let Some(sender) = sender {
            if sender.send(message).is_err() {
                warn!(
                    "Error replying message to request {}. Channel got dropped.",
                    request_id
                );
            }
        } else {
            warn!(
                "Tried to reply to request {}, but wasn't there anymore (timed-out?)",
                request_id
            );
        }
    }

    pub async fn remove(&self, request_id: RequestID) {
        let mut requests = self.requests.lock().await;
        requests.remove(&request_id);
    }
}

/// Receiving end of a the tracked request. This is used in the HTTP request
/// handler to wait for a reply from a service.
pub struct TrackedRequest {
    id: RequestID,
    requests: Weak<RequestTracker>,
    receiver: Option<oneshot::Receiver<OutMessage>>,
    receive_timeout: Duration,
}

impl TrackedRequest {
    pub fn id(&self) -> RequestID {
        self.id
    }

    pub async fn get_response_or_timeout(mut self) -> Result<OutMessage, ()> {
        let receiver = self.receiver.take().ok_or(())?;
        let timeout = delay_for(self.receive_timeout);

        futures::select! {
            resp = receiver.fuse() => {
                resp.map_err(|_| ())
            },
            _ = timeout.fuse() => {
                Err(())
            },
        }
    }
}

impl Drop for TrackedRequest {
    fn drop(&mut self) {
        if let Some(requests) = self.requests.upgrade() {
            block_on(requests.remove(self.id));
        }
    }
}
