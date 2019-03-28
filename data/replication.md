
# Replication
The replication is handled by [`Engine`](src/engine/mod.rs).
 
There are 2 data structures to replicate:
* **Chain**: immutable collection of blocks that contain entries. Entries can be arbitrary data stored for the index layer,
         or metadata entries related to the chain.
         
* **Pending store**: transient store in which latest operations are aggregated, to eventually be added to the chain.

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
    * OperationEntryNew
* Block related (group id = block id)
    * BlockPropose
    * BlockProposalSign
    * BlockProposalRefuse (can happen after sign if node detects anomaly or accepts a better block)
* Maintenance related
    * Pending store cleanup mark (TODO)

### Cleanup
* We should only cleanup if stuff were committed to the chain OR we got a refusal quorum (everybody refused something).
* If a node was offline and received data before cleanup point, it will eventually get deleted if it had been put in chain already.

## Chain replication
TODO

### Messages
* ChainSyncRequest

### Cleanup
* A node that has access to unencrypted data can decide to cleanup the chain by truncating it, after moving entries around.
  The process:
  * Iterate through old blocks
  * For each entry, check if it's an old version of an entry
  * If it's an old entry, add to pending
  * Once we have a part of a chain that contains only old versions, propose a chain truncation

## Exceptions
* A node has signature of other nodes on a block, and is about to send his signature, but then get partitioned.
  He's the only one with quorum, and adds to the block.

  Solutions:
  * He'll never be able to make progress since all other nodes will eventually timeout and commit another block.
    He'll have to truncate its chain once he's sync back.
    Cons: We may be losing? Not supposed, since they will still be in other node's pending

  * Two stage commit where nobody adds to the chain unless everybody has agreed that they have signatures.
    Cons: This adds latency and communication for nothing... And it's an never ending story.


## TODO
- [ ] What is the logic on who proposes
        * Needs to have full data access
        * Needs to be considered online by others for them to wait for its proposal
- [ ] Conditional entry: entry can be conditional on time, other entry commit, etc.