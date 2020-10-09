use super::{EngineError, Inner};
use crate::block::{Block, BlockHeight, BlockOffset, BlockOwned, BlockRef};
use crate::operation::{OperationBuilder, OperationId};
use crate::pending;
use crate::pending::CommitStatus;
use crate::{chain, operation};
use exocore_core::protos::generated::data_chain_capnp::chain_operation;
use exocore_core::utils::handle_set::Handle;
use futures::prelude::*;
use std::ops::RangeBounds;
use std::sync::{Arc, RwLock, Weak};

/// Handle ot the Engine, allowing communication with the engine.
/// The engine itself is owned by a future executor.
pub struct EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    inner: Weak<RwLock<Inner<CS, PS>>>,
    handle: Handle,
}

impl<CS, PS> EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    pub(crate) fn new(inner: Weak<RwLock<Inner<CS, PS>>>, handle: Handle) -> EngineHandle<CS, PS> {
        EngineHandle { inner, handle }
    }

    pub fn on_started(&self) -> impl Future<Output = ()> {
        self.handle.on_set_started()
    }

    pub fn write_entry_operation(&self, data: &[u8]) -> Result<OperationId, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let mut unlocked_inner = inner.write()?;

        let my_node = unlocked_inner.cell.local_node();
        let operation_id = unlocked_inner.clock.consistent_time(my_node).into();

        let operation_builder = OperationBuilder::new_entry(operation_id, my_node.id(), data);
        let operation = operation_builder.sign_and_build(&my_node)?;

        unlocked_inner.handle_new_operation(operation)?;

        Ok(operation_id)
    }

    pub fn get_chain_segments(&self) -> Result<chain::Segments, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        Ok(unlocked_inner.chain_store.segments())
    }

    pub fn get_chain_operation(
        &self,
        block_offset: BlockOffset,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let block = unlocked_inner.chain_store.get_block(block_offset)?;
        EngineOperation::from_chain(block, operation_id)
    }

    pub fn get_chain_operations(
        &self,
        from_offset: Option<BlockOffset>,
    ) -> ChainOperationsIterator<CS, PS> {
        ChainOperationsIterator::new(self.inner.clone(), from_offset)
    }

    pub fn get_chain_last_block_info(
        &self,
    ) -> Result<Option<(BlockOffset, BlockHeight)>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        let last_block = unlocked_inner.chain_store.get_last_block()?;

        if let Some(last_block) = last_block {
            let height = last_block.get_height()?;
            Ok(Some((last_block.offset, height)))
        } else {
            Ok(None)
        }
    }

    pub fn get_chain_block_info(
        &self,
        offset: BlockOffset,
    ) -> Result<Option<(BlockOffset, BlockHeight)>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        let block = unlocked_inner.chain_store.get_block(offset).ok();

        if let Some(block) = block {
            let height = block.get_height()?;
            Ok(Some((block.offset, height)))
        } else {
            Ok(None)
        }
    }

    pub fn get_chain_block(&self, offset: BlockOffset) -> Result<Option<BlockOwned>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;
        let block = unlocked_inner.chain_store.get_block(offset).ok();

        if let Some(block) = block {
            Ok(Some(block.to_owned()))
        } else {
            Ok(None)
        }
    }

    pub fn get_pending_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operation = unlocked_inner
            .pending_store
            .get_operation(operation_id)?
            .map(EngineOperation::from_pending);

        Ok(operation)
    }

    pub fn get_pending_operations<R: RangeBounds<OperationId>>(
        &self,
        operations_range: R,
    ) -> Result<Vec<EngineOperation>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        let operations = unlocked_inner
            .pending_store
            .operations_iter(operations_range)?
            .map(EngineOperation::from_pending)
            .collect::<Vec<_>>();
        Ok(operations)
    }

    pub fn get_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let unlocked_inner = inner.read()?;

        // first check if it's in pending store with a clear commit status
        let pending_operation = unlocked_inner.pending_store.get_operation(operation_id)?;
        let pending_operation = if let Some(pending_operation) = pending_operation {
            if pending_operation.commit_status != CommitStatus::Unknown {
                return Ok(Some(EngineOperation::from_pending(pending_operation)));
            }

            Some(pending_operation)
        } else {
            None
        };

        // if it's not found in pending store, or that it didn't have a clear status, we
        // check in chain
        if let Some(block) = unlocked_inner
            .chain_store
            .get_block_by_operation_id(operation_id)?
        {
            if let Some(chain_operation) = EngineOperation::from_chain(block, operation_id)? {
                return Ok(Some(chain_operation));
            }
        }

        // if we're here, the operation was either absent, or just had a unknown status
        // in pending store we return the pending store operation (if any)
        Ok(pending_operation.map(EngineOperation::from_pending))
    }

    /// Take the events stream receiver out of this `Handle`.
    /// This stream is bounded and consumptions should be non-blocking to
    /// prevent losing events. Calling the engine on every call should be
    /// throttled in the case of a big read amplification.
    pub fn take_events_stream(&mut self) -> Result<impl Stream<Item = Event>, EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let mut unlocked_inner = inner.write()?;

        let stream = unlocked_inner.get_new_events_stream(self.handle.id());

        Ok(stream)
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
                unlocked_inner.unregister_handle(self.handle.id());
            }
        }
    }
}

impl<CS, PS> Clone for EngineHandle<CS, PS>
where
    CS: chain::ChainStore,
    PS: pending::PendingStore,
{
    fn clone(&self) -> Self {
        EngineHandle::new(self.inner.clone(), self.handle.clone())
    }
}

/// Events dispatched to handles to notify changes in the different stores.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The engine is now started
    Started,

    /// The stream of events hit the maximum buffer size, and some events got
    /// discarded. Consumer state should be rebuilt from scratch to prevent
    /// having inconsistencies.
    StreamDiscontinuity,

    /// An operation added to the pending store.
    NewPendingOperation(OperationId),

    /// A new block got added to the chain.
    NewChainBlock(BlockOffset),

    /// The chain has diverged from given offset, which mean it will get
    /// re-written with new blocks. Operations after this offset should
    /// ignored.
    ChainDiverged(BlockOffset),
}

/// Operation that comes either from the chain or from the pending store
pub struct EngineOperation {
    pub operation_id: OperationId,
    pub status: EngineOperationStatus,
    pub operation_frame: Arc<super::operation::OperationFrame<Vec<u8>>>,
}

impl EngineOperation {
    fn from_pending(operation: pending::StoredOperation) -> EngineOperation {
        let status = match operation.commit_status {
            pending::CommitStatus::Committed(offset, height) => {
                EngineOperationStatus::Committed(offset, height)
            }
            _ => EngineOperationStatus::Pending,
        };

        EngineOperation {
            operation_id: operation.operation_id,
            status,
            operation_frame: operation.frame,
        }
    }

    fn from_chain(
        block: BlockRef,
        operation_id: OperationId,
    ) -> Result<Option<EngineOperation>, EngineError> {
        if let Some(operation) = block.get_operation(operation_id)? {
            let height = block.get_height()?;
            return Ok(Some(EngineOperation {
                operation_id,
                status: EngineOperationStatus::Committed(block.offset, height),
                operation_frame: Arc::new(operation.to_owned()),
            }));
        }

        Ok(None)
    }
}

impl crate::operation::Operation for EngineOperation {
    fn get_operation_reader(&self) -> Result<chain_operation::Reader, operation::Error> {
        Ok(self.operation_frame.get_reader()?)
    }
}

#[derive(Debug, PartialEq)]
pub enum EngineOperationStatus {
    Committed(BlockOffset, BlockHeight),
    Pending,
}

impl EngineOperationStatus {
    pub fn is_committed(&self) -> bool {
        matches!(self, EngineOperationStatus::Committed(_offset, _height))
    }
}

/// Iterator of operations in the chain
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

    fn fetch_next_block(&mut self) -> Result<(), EngineError> {
        let inner = self.inner.upgrade().ok_or(EngineError::InnerUpgrade)?;
        let inner = inner.read()?;

        // since a block may not contain operations (ex: genesis), we need to loop until
        // we find one
        while self.current_operations.is_empty() {
            let block = inner.chain_store.get_block(self.next_offset)?;
            let height = block.get_height()?;

            for operation in block.operations_iter()? {
                let operation_reader = operation.get_reader()?;
                let operation_id = operation_reader.get_operation_id();

                self.current_operations.push(EngineOperation {
                    operation_id,
                    status: EngineOperationStatus::Committed(block.offset, height),
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
            if let Err(EngineError::ChainStore(chain::Error::OutOfBound(_))) =
                self.fetch_next_block()
            {
                return None;
            }
        }

        self.current_operations.pop()
    }
}
