use super::store::StoreConfig;
use crate::error::Error;
use exocore_chain::operation::OperationId;
use exocore_core::protos::index::MutationResult;
use exocore_core::time::Instant;
use futures::channel::oneshot;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Mutex;

type RequestId = usize;

pub(crate) struct WatchedMutations {
    inner: Mutex<Inner>,
}

impl WatchedMutations {
    pub fn new(config: StoreConfig) -> WatchedMutations {
        WatchedMutations {
            inner: Mutex::new(Inner {
                config,
                next_request_id: 0,
                requests: HashMap::new(),
                operations_requests: HashMap::new(),
            }),
        }
    }

    pub fn track_request(
        &self,
        operation_ids: Vec<OperationId>,
        response_channel: oneshot::Sender<Result<MutationResult, Error>>,
    ) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();

        let request_id = inner.next_request_id;
        inner.next_request_id += 1;

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

        Ok(())
    }

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
                let operation_ids = Vec::from_iter(request.operation_ids.iter().cloned());
                let res = request.response_channel.send(Ok(MutationResult {
                    operation_ids,
                    entity: None,
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
        let request = if let Some(request) = self.requests.remove(&request_id) {
            request
        } else {
            return None;
        };

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
            if request.received_time.elapsed() > self.config.watched_mutations_timeout {
                warn!(
                    "Tracked mutations for operations {:?} timed out",
                    request.operation_ids
                );
                expired_requests.push(*request_id);
            }
        }

        for request_id in expired_requests {
            self.remove_request(request_id);
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

    #[test]
    fn test_track_request() {}
}
