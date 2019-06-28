use super::{Error, Inner};
use crate::block::{Block, BlockOffset, BlockRef};
use crate::operation::{OperationBuilder, OperationId};
use crate::pending;
use crate::pending::CommitStatus;
use crate::{chain, operation};
use exocore_common;
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::utils::completion_notifier::{CompletionError, CompletionListener};
use futures::prelude::*;
use futures::sync::mpsc;
use std::ops::RangeBounds;
use std::sync::{Arc, RwLock, Weak};

///
/// Handle ot the Engine, allowing communication with the engine.
/// The engine itself is owned by a future executor.
///
pub struct EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    id: usize,
    inner: Weak<RwLock<Inner<CS, PS>>>,
    events_receiver: Option<mpsc::Receiver<Event>>,
    start_listener: CompletionListener<(), Error>,
    stop_listener: CompletionListener<(), Error>,
}

impl<CS, PS> EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub(crate) fn new(
        id: usize,
        inner: Weak<RwLock<Inner<CS, PS>>>,
        events_receiver: Option<mpsc::Receiver<Event>>,
        start_listener: CompletionListener<(), Error>,
        stop_listener: CompletionListener<(), Error>,
    ) -> EngineHandle<CS, PS> {
        EngineHandle {
            id,
            inner,
            events_receiver,
            start_listener,
            stop_listener,
        }
    }

    pub fn on_start(&self) -> Result<impl Future<Item = (), Error = Error>, Error> {
        Ok(self
            .start_listener
            .try_clone()
            .map_err(|_err| Error::Other("Couldn't clone start listener in handle".to_string()))?
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            }))
    }

    pub fn on_stop(&self) -> Result<impl Future<Item = (), Error = Error>, Error> {
        Ok(self
            .stop_listener
            .try_clone()
            .map_err(|_err| Error::Other("Couldn't clone stop listener in handle".to_string()))?
            .map_err(|err| match err {
                CompletionError::UserError(err) => err,
                _ => Error::Other("Error in completion error".to_string()),
            }))
    }

    pub fn write_entry_operation(&self, data: &[u8]) -> Result<OperationId, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let mut unlocked_inner = inner.write()?;

        let my_node = unlocked_inner.cell.local_node();
        let operation_id = unlocked_inner.clock.consistent_time(my_node);

        let operation_builder = OperationBuilder::new_entry(operation_id, my_node.id(), data);
        let operation = operation_builder.sign_and_build(&my_node)?;

        unlocked_inner.handle_add_pending_operation(operation)?;

        Ok(operation_id)
    }

    pub fn get_chain_segments(&self) -> Result<Vec<chain::Segment>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        Ok(unlocked_inner.chain_store.segments())
    }

    pub fn get_chain_operation(
        &self,
        block_offset: BlockOffset,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let block = unlocked_inner.chain_store.get_block(block_offset)?;
        EngineOperation::from_chain_block(block, operation_id)
    }

    pub fn get_chain_operations(
        &self,
        from_offset: Option<BlockOffset>,
    ) -> ChainOperationsIterator<CS, PS> {
        ChainOperationsIterator::new(self.inner.clone(), from_offset)
    }

    pub fn get_pending_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operation = unlocked_inner
            .pending_store
            .get_operation(operation_id)?
            .map(EngineOperation::from_pending_operation);

        Ok(operation)
    }

    pub fn get_pending_operations<R: RangeBounds<OperationId>>(
        &self,
        operations_range: R,
    ) -> Result<Vec<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operations = unlocked_inner
            .pending_store
            .operations_iter(operations_range)?
            .map(EngineOperation::from_pending_operation)
            .collect::<Vec<_>>();
        Ok(operations)
    }

    pub fn get_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        // first check if it's in pending store with a clear commit status
        let pending_operation = unlocked_inner.pending_store.get_operation(operation_id)?;
        let pending_operation = if let Some(pending_operation) = pending_operation {
            if pending_operation.commit_status != CommitStatus::Unknown {
                return Ok(Some(EngineOperation::from_pending_operation(
                    pending_operation,
                )));
            }

            Some(pending_operation)
        } else {
            None
        };

        // if it's not found in pending store, or that it didn't have a clear status, we check in chain
        if let Some(block) = unlocked_inner
            .chain_store
            .get_block_by_operation_id(operation_id)?
        {
            if let Some(chain_operation) = EngineOperation::from_chain_block(block, operation_id)? {
                return Ok(Some(chain_operation));
            }
        }

        // if we're here, the operation was either absent, or just had a unknown status in pending store
        // we return the pending store operation (if any)
        Ok(pending_operation.map(EngineOperation::from_pending_operation))
    }

    ///
    /// Take the events stream receiver out of this `Handle`.
    /// This stream is bounded and consumptions should be non-blocking to prevent losing events.
    /// Calling the engine on every call should be throttled in the case of a big read amplification.
    pub fn take_events_stream(&mut self) -> impl futures::Stream<Item = Event, Error = Error> {
        self.events_receiver
            .take()
            .expect("Get events stream was already called.")
            .map_err(|_err| Error::Other("Error in incoming events stream".to_string()))
    }
}

impl<CS, PS> Drop for EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn drop(&mut self) {
        debug!("Engine handle got dropped.");

        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut unlocked_inner) = inner.write() {
                unlocked_inner.unregister_handle(self.id);
            }
        }
    }
}

///
/// Events dispatched to handles to notify changes in the different stores.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The engine is now started
    Started,

    /// The stream of events hit the maximum buffer size, and some events got discarded.
    /// Consumer state should be rebuilt from scratch to prevent having inconsistencies.
    StreamDiscontinuity,

    /// An operation added to the pending store.
    PendingOperationNew(OperationId),

    /// An operation that was previously added got deleted, hence will never end up in a block.
    /// This happens if an operation was invalid or found in the chain later on.
    PendingEntryDelete(OperationId),

    /// A new block got added to the chain.
    ChainBlockNew(BlockOffset),

    /// The chain has diverged from given offset, which mean it will get re-written with new blocks.
    /// Operations after this offset should ignored.
    ChainDiverged(BlockOffset),
}

///
/// Operation that comes either from the chain or from the pending store
///
pub struct EngineOperation {
    pub operation_id: OperationId,
    pub status: EngineOperationStatus,
    pub operation_frame: Arc<super::operation::OperationFrame<Vec<u8>>>,
}

impl EngineOperation {
    fn from_pending_operation(operation: pending::StoredOperation) -> EngineOperation {
        let status = match operation.commit_status {
            pending::CommitStatus::Committed(offset, _depth) => {
                EngineOperationStatus::Committed(offset)
            }
            _ => EngineOperationStatus::Pending,
        };

        EngineOperation {
            operation_id: operation.operation_id,
            status,
            operation_frame: operation.frame,
        }
    }

    fn from_chain_block(
        block: BlockRef,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, Error> {
        if let Some(operation) = block.get_operation(operation_id)? {
            return Ok(Some(EngineOperation {
                operation_id,
                status: EngineOperationStatus::Committed(block.offset),
                operation_frame: Arc::new(operation.to_owned()),
            }));
        }

        Ok(None)
    }
}

impl crate::operation::Operation for EngineOperation {
    fn get_operation_reader(&self) -> Result<pending_operation::Reader, operation::Error> {
        Ok(self.operation_frame.get_reader()?)
    }
}

#[derive(Debug, PartialEq)]
pub enum EngineOperationStatus {
    Committed(BlockOffset),
    Pending,
}

impl EngineOperationStatus {
    pub fn is_committed(&self) -> bool {
        match self {
            EngineOperationStatus::Committed(_offset) => true,
            _ => false,
        }
    }
}

///
/// Iterator of operations in the chain
///
pub struct ChainOperationsIterator<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    next_offset: BlockOffset,
    current_operations: Vec<EngineOperation>,
    inner: Weak<RwLock<Inner<CS, PS>>>,
}

impl<CS, PS> ChainOperationsIterator<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn new(
        inner: Weak<RwLock<Inner<CS, PS>>>,
        from_offset: Option<BlockOffset>,
    ) -> ChainOperationsIterator<CS, PS> {
        ChainOperationsIterator {
            next_offset: from_offset.unwrap_or(0),
            current_operations: Vec::new(),
            inner,
        }
    }

    fn fetch_next_block(&mut self) -> Result<(), Error> {
        let inner = self.inner.upgrade().ok_or(Error::InnerUpgrade)?;
        let inner = inner.read()?;

        // since a block may not contain operations (ex: genesis), we need to loop until we find one
        while self.current_operations.is_empty() {
            let block = inner.chain_store.get_block(self.next_offset)?;
            for operation in block.operations_iter()? {
                let operation_reader = operation.get_reader()?;
                let operation_id = operation_reader.get_operation_id();

                self.current_operations.push(EngineOperation {
                    operation_id,
                    status: EngineOperationStatus::Committed(block.offset),
                    operation_frame: Arc::new(operation.to_owned()),
                });
            }

            // need to reverse as we will pop from end
            self.current_operations.reverse();
            self.next_offset = block.next_offset();
        }

        Ok(())
    }
}

impl<CS, PS> Iterator for ChainOperationsIterator<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    type Item = EngineOperation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_operations.is_empty() {
            if let Err(Error::ChainStore(chain::Error::OutOfBound(_))) = self.fetch_next_block() {
                return None;
            }
        }

        self.current_operations.pop()
    }
}
