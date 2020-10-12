use super::store::StoreConfig;
use crate::error::Error;
use exocore_chain::operation::OperationId;
use exocore_core::protos::store::MutationResult;
use exocore_core::time::Instant;
use futures::channel::oneshot;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Mutex;

type RequestId = usize;

/// Tracks mutations for which the user requested to be notified when they have
/// been indexed and are reflected in the store.
///
/// This tracks operation ids that have been indexed, and sends completion
/// signal via the provided oneshot channel.
pub(crate) struct MutationTracker {
    inner: Mutex<Inner>,
}

impl MutationTracker {
    /// Creates a new mutation tracker.
    pub fn new(config: StoreConfig) -> MutationTracker {
        MutationTracker {
            inner: Mutex::new(Inner {
                config,
                next_request_id: 0,
                requests: HashMap::new(),
                operations_requests: HashMap::new(),
            }),
        }
    }

    /// Track the indexation status of the given operation ids and notify they
    /// indexation completion or timeout via the given channel.
    pub fn track_request(
        &self,
        operation_ids: Vec<OperationId>,
        response_channel: oneshot::Sender<Result<MutationResult, Error>>,
    ) {
        let mut inner = self.inner.lock().unwrap();

        let request_id = inner.next_request_id;
        inner.next_request_id += 1;

        debug!(
            "Tracking operations={:?} with request id={}",
            operation_ids, request_id
        );

        inner.requests.insert(
            request_id,
            WatchedMutationRequest {
                received_time: Instant::now(),
                operation_ids: HashSet::from_iter(operation_ids.iter().cloned()),
                completed_operation_ids: HashSet::new(),
                response_channel,
            },
        );

        for operation_id in operation_ids {
            let entry = inner
                .operations_requests
                .entry(operation_id)
                .or_insert_with(HashSet::new);
            entry.insert(request_id);
        }
    }

    /// Notifies that the given operation ids were indexed and available through
    /// the store.
    pub fn handle_indexed_operations(&self, operation_ids: &[OperationId]) {
        let mut inner = self.inner.lock().unwrap();

        let mut completed_requests = HashSet::new();
        for operation_id in operation_ids {
            if let Some(request_ids) = inner.operations_requests.remove(operation_id) {
                for req_id in &request_ids {
                    if let Some(request) = inner.requests.get_mut(req_id) {
                        request.completed_operation_ids.insert(*operation_id);

                        if request.completed_operation_ids.len() == request.operation_ids.len() {
                            completed_requests.insert(*req_id);
                        }
                    }
                }
            }
        }

        for request_id in completed_requests {
            if let Some(request) = inner.remove_request(request_id) {
                debug!("Request id={} completed", request_id);

                let operation_ids = Vec::from_iter(request.operation_ids.iter().cloned());
                let res = request.response_channel.send(Ok(MutationResult {
                    operation_ids,
                    ..Default::default()
                }));

                if res.is_err() {
                    error!("Error sending response to watched mutation. Sender was dropped");
                }
            }
        }

        inner.cleanup_expired();
    }
}

struct Inner {
    config: StoreConfig,
    requests: HashMap<RequestId, WatchedMutationRequest>,
    operations_requests: HashMap<OperationId, HashSet<RequestId>>,
    next_request_id: RequestId,
}

impl Inner {
    fn remove_request(&mut self, request_id: RequestId) -> Option<WatchedMutationRequest> {
        let request = self.requests.remove(&request_id)?;
        for operation_id in &request.operation_ids {
            if let Some(op_reqs) = self.operations_requests.get_mut(operation_id) {
                op_reqs.remove(&request_id);
            }
        }

        Some(request)
    }

    fn cleanup_expired(&mut self) {
        let mut expired_requests = Vec::new();
        for (request_id, request) in &self.requests {
            if request.received_time.elapsed() > self.config.mutation_tracker_timeout {
                warn!(
                    "Tracked mutations for operations {:?} timed out after {:?}",
                    request.operation_ids,
                    request.received_time.elapsed(),
                );
                expired_requests.push(*request_id);
            }
        }

        if expired_requests.is_empty() {
            return;
        }

        for request_id in expired_requests {
            self.remove_request(request_id);
        }

        let mut orphan_operations = Vec::new();
        for (operation_id, requests) in &self.operations_requests {
            if requests.is_empty() {
                orphan_operations.push(*operation_id);
            }
        }

        for operation_id in orphan_operations {
            self.operations_requests.remove(&operation_id);
        }
    }
}

pub struct WatchedMutationRequest {
    received_time: Instant,
    operation_ids: HashSet<OperationId>,
    completed_operation_ids: HashSet<OperationId>,
    response_channel: oneshot::Sender<Result<MutationResult, Error>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn golden_path() {
        let tracker = MutationTracker::new(Default::default());

        let (sender, mut receiver) = oneshot::channel();
        tracker.track_request(vec![1, 2, 3], sender);
        tracker.handle_indexed_operations(&[1, 2, 3]);
        assert!(receiver.try_recv().unwrap().is_some());

        let (sender, mut receiver) = oneshot::channel();
        tracker.track_request(vec![4, 5, 6], sender);
        tracker.handle_indexed_operations(&[4, 5]);
        assert!(receiver.try_recv().unwrap().is_none());
        tracker.handle_indexed_operations(&[6, 7]);
        assert!(receiver.try_recv().unwrap().is_some());

        {
            let inner = tracker.inner.lock().unwrap();
            assert_eq!(inner.operations_requests.len(), 0);
            assert_eq!(inner.requests.len(), 0);
        }
    }

    #[test]
    fn tracking_timeout() {
        let mutation_tracker_timeout = Duration::from_millis(1);
        let tracker = MutationTracker::new(StoreConfig {
            mutation_tracker_timeout,
            ..Default::default()
        });

        let (sender, mut receiver) = oneshot::channel();
        tracker.track_request(vec![1, 2, 3], sender);

        std::thread::sleep(mutation_tracker_timeout);
        tracker.handle_indexed_operations(&[]);

        assert!(receiver.try_recv().is_err());

        {
            let inner = tracker.inner.lock().unwrap();
            assert_eq!(inner.operations_requests.len(), 0);
            assert_eq!(inner.requests.len(), 0);
        }
    }

    #[test]
    fn multiple_requests_same_operation() {
        let tracker = MutationTracker::new(Default::default());

        let (sender1, mut receiver1) = oneshot::channel();
        tracker.track_request(vec![1, 2, 3], sender1);

        let (sender2, mut receiver2) = oneshot::channel();
        tracker.track_request(vec![1, 2, 3, 4], sender2);

        tracker.handle_indexed_operations(&[1, 2, 3]);

        assert!(receiver1.try_recv().unwrap().is_some());
        assert!(receiver2.try_recv().unwrap().is_none());

        tracker.handle_indexed_operations(&[4, 5]);

        assert!(receiver2.try_recv().unwrap().is_some());
    }
}
