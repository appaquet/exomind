use exocore_chain::operation::OperationId;
use exocore_core::protos::index::MutationResult;
use exocore_core::time::Instant;
use futures::channel::oneshot;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Mutex;
use crate::error::Error;

// TODO: Timeout

type RequestId = u64;

pub(crate) struct TrackedMutations {
    inner: Mutex<Inner>,
}

impl TrackedMutations {
    pub fn new() -> TrackedMutations {
        TrackedMutations {
            inner: Mutex::new(Inner {
                next_id: 0,
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

        let request_id = inner.next_id;
        inner.next_id += 1;

        let tracked_mutation = WatchedMutationRequest {
            request_id,
            received_time: Instant::now(),
            operation_ids: HashSet::from_iter(operation_ids.iter().cloned()),
            completed_operation_ids: HashSet::new(),
            response_channel,
        };

        for operation_id in tracked_mutation.operation_ids.iter() {
            inner.operations_requests.insert(*operation_id, request_id);
        }

        inner.requests.insert(request_id, tracked_mutation);

        Ok(())
    }

    pub fn handle_indexed_operations(&self, operation_ids: &[OperationId]) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();

        let mut completed_requests = HashSet::new();
        for operation_id in operation_ids {
            if let Some(req_id) = inner.operations_requests.remove(operation_id) {
                if let Some(request) = inner.requests.get_mut(&req_id) {
                    request.completed_operation_ids.insert(req_id);

                    if request.completed_operation_ids.len() == request.operation_ids.len() {
                        completed_requests.insert(req_id);
                    }
                }
            }
        }

        for req_id in completed_requests {
            if let Some(request) = inner.requests.remove(&req_id) {
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

        Ok(())
    }
}

struct Inner {
    next_id: u64,
    requests: HashMap<RequestId, WatchedMutationRequest>,
    operations_requests: HashMap<OperationId, RequestId>,
}

pub(crate) struct WatchedMutationRequest {
    request_id: RequestId,
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
