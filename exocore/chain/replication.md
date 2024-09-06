
# Replication
The replication is handled by [`Engine`](src/engine/mod.rs).
 
There are 2 data structures to replicate:
* **Pending store**
    * Transient store in which latest operations are stored, to eventually be added to the chain by the commit manager. 
    * Operations are kept in the pending store even if they are committed to the chain, but eventually gets cleaned up.

* **Chain**: 
    * Immutable collection of blocks that each contains operations. Operations are the same as in the pending store.
    * A chain structure is used to make replication easier between untrusted nodes.

Synchronization for each structure is managed independently. The chain is synchronized by the [`Chain Synchronizer`](src/engine/chain_sync/mod.rs),
while the pending store is replicated by the [`Pending Store Synchronizer`](src/engine/pending_sync/mod.rs).

Once operations are stored in the pending store, the [`Commit manager`](src/engine/commit_manager/mod.rs) proposes a block to be 
added to a specific offset of the chain. This block can then be signed/voted by other nodes, or refused. If a proposed block
receives enough signatures, it's then added to the local chain by each node.

## Pending store replication
The pending store's replication is managed by the [`Pending Store Synchronizer`](src/engine/pending_sync/mod.rs).

### Messages
* `PendingSyncRequest(List<PendingSyncRange>)`
    * Maximum depth in which requests can be committed
      * This is used to prevent nodes from synchronizing operations that are now considered committed and could be cleaned up from
        the pending store.
    * Each `PendingSyncRange` contains:
      * The bounds of the range of operations to compare. Bounds can be omitted to represent no boundaries (represented by value 0)
      * The metadata information of that range in the local store (hash + count)
      * Operations data that need to be applied before comparing the local store information
      * Operations headers that are given to compare the local store's operations, resulting in the stores to request or send
        missing operations to the other nodes.

#### Example:
```
   A                               B
 0,5,10                          0,5,12
   |                               |
   |----meta(-∞..5)+meta(6..∞)---->| (meta is count+hash)
   |                               |
   |<---meta(-∞..5)+head(6..∞)-----| (head are all headers of operations)
   |                               |
   |----meta(-∞..5)+data(10)------>| (sends data of 10 + headers of rest of range)
   |                               |
   |<---meta(-∞..5)+data(12)-------| (sends data of 12 + headers of rest of range)
   |                               |
   X                               | (stops, because ranges have same hash+count)
```

#### Operations
* Each operation contains an unique operation id, which is globally unique and monotonically increasing.
* Each operation has a group id, which is used to combined related operations.
* Per example, operations related to a single block have the same group id, which is the operation id of the block proposal.
* Operations in pending store can be:
    * Data entries (group id = entry id)
        * OperationEntry
    * Block related operations (group id = block id)
        * OperationBlockPropose
        * OperationBlockSign
        * OperationBlockRefuse (can happen after sign if node detects anomaly or accepts a better block)

### Cleanup
* We should only cleanup if operations were committed at a configurable depth or more.
* When a node gets partitioned, it could revive old operations. 
  To prevent that, the chain synchronizer always needs to run first. By doing this, 
  the node will figure out that operations that are in its local pending store need 
  to be cleaned.
* The pending synchronizer synchronization requests specify a maximum depth that we should include
  pending operations if they were committed. This depth should ideally be kept smaller than the pending
  store cleaning depth so that we don't revive operations that got cleaned up on another node.

## Chain replication
The chain's replication is managed by the [`Chain Synchronizer`](src/engine/chain_sync/mod.rs).

### Messages
* `ChainSyncRequest`
  * Offsets range of the lookup
  * Requested details (blocks headers or full data)
* `ChainSyncResponse`
  * Blocks headers or full data for matching request


## Commit management
At interval, the [`Commit Manager`](src/engine/commit_manager/mod.rs) on each node checks if new blocks could
be committed to the chain from uncommitted operations in the chain. Based on the consistent time, each node can
turn by turn propose new blocks. This proposal is done via the operations stored in the pending store.

When other nodes receive proposal, they can either approve it by signing it, or refuse it. Reasons of refusal could
be that one operation was already committed previously, or that the block hash is invalid.

At interval, the commit manager is also cleaning up operations from the pending store that are at a certain depth.

## Exceptions
* A node has signature of other nodes on a block, and is about to send his signature, but then get partitioned.
  It's the only one who had enough signatures for quorum, and commits to the block locally.

  Solutions:
  * It will never be able to make progress since all other nodes will eventually timeout and commit another block.
    It will have to truncate its chain once he syncs back with the rest of the nodes.
    Data won't be lost since operations are kept in pending store until they are in a block with certain depth.
    If the node had operations that only itself had, they will be sent back since they will still be in its pending store.

* A node local chain changes after we synced against it.

  Solution: 
  * The chain synchronizer will figure this out since the metadata of that node will not match anymore

* A node that hasn't synchronized for a while may bring old items in the pending store.

  Solution:
  * A node that comes only always synchronize its chain first, so operations that needs to be deleted will be in the chain.
    After the chain is synchronized, the commit manager is always called first so that it can clean up operations that are
    at more than a certain depth. The old items could therefor get cleaned.
  
* If multiple nodes get added to the cell at once, they may change the quorum need, but they cannot participate yet...

  Solutions:
  * A node that boots needs to be considered a data node at first so that it doesn't prevent quorum.
  * A node may need to have a flag that indicate that it synchronized once, so that we can ignore them in quorum.


