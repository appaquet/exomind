
# Replication
The replication is handled by [`Engine`](src/engine/mod.rs).
 
There are 2 data structures to replicate:
* **Pending store**: transient store in which latest operations are stored, to eventually be added to the chain by the commit manager.

* **Chain**: immutable collection of blocks that each contains operations. Operations are the same as in the pending store.

Synchronization for each structure is handled independently. The chain is synchronized using the [`Chain Synchronizer`](src/engine/chain_sync.rs),
while the pending store is replicated using the [`Pending Store Synchronizer`](src/engine/pending_sync.rs).

Once operations are stored in the pending store, the [`Commit manager`](src/engine/commit_manager.rs) proposes a block to be 
added to a specific offset of the chain. This block can then be signed/voted by other nodes, or refused. If a proposed block
receives enough signatures, it's then added to the local chain by each node.



## Pending store replication
Pending store's replication is handled by the [`Pending Store Synchronizer`](src/engine/pending_sync.rs).

### Messages
* `PendingSyncRequest(List<PendingSyncRange>)`
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
Each operation contains an unique operation ID, which is globally unique and monotonically increasing.
Each operation has a group ID, which is used to combined related operations.
Per example, operations related to a single block have the same group ID, which is the Operation ID of the block proposal.

Operations in pending store can be:

* Entries related (group id = entry id)
    * OperationEntry
* Block related (group id = block id)
    * OperationBlockPropose
    * OperationBlockSign
    * OperationBlockRefuse (can happen after sign if node detects anomaly or accepts a better block)
* Maintenance related
    * PendingIgnore (used to delete pending store items that weren't committed to a block because of their invalidity)

### Cleanup
* We should only cleanup if stuff were committed to the chain OR we got a refusal quorum (everybody refused something).
* If a node was offline and received data before cleanup point, it will eventually get deleted if it had been put in chain already.
* Cleanup is done once operations have reached a certain height in the chain.



## Chain replication
Chain's replication is handled by the [`Chain Synchronizer`](src/engine/chain_sync.rs).

### Messages
* ChainSyncRequest
* ChainSyncResponse

### Cleanup
* A node that has access to unencrypted data can decide to cleanup the chain by truncating it, after moving entries around.
  The process:
  * Iterate through old blocks
  * For each entry, check if it's an old version of an entry
  * If it's an old entry, add to pending
  * Once we have a part of a chain that contains only old versions, propose a chain truncation



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
    We could then add a initialization step that requires the commit manager to cleanup old stuff before doing a first pending
    store synchronization.
  
* If multiple nodes get added to the cell at once, they may change the quorum need, but they cannot participate yet...

  Solutions:
  * A node that boots needs to be considered a data node at first so that it doesn't prevent quorum.
  * A node may need to have a flag that indicate that it synchronized once, so that we can ignore them in quorum.


