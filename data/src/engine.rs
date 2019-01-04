use crate::chain;
use crate::pending;
use crate::transport;

use exocore_common;

use tokio;

use std::time::Instant;

// TODO: Should have a "EngineState" so that we can easily test the states transition / actions
// TODO: If node has access to data, it needs ot check its integrity by the upper layer
// TODO: If not, a node needs to wait for a majority of nodes that has data

pub struct Engine<T, CP, PP>
where
    T: transport::Transport,
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    runtime: tokio::runtime::Runtime, // TODO: Should it have its own Runtime? Or just just be a future itself
    transport: T,
    nodes: Vec<exocore_common::node::Node>,
    chain: chain::Chain<CP>,
    pending: pending::Store<PP>,
}

impl<T, CP, PP> Engine<T, CP, PP>
where
    T: transport::Transport,
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    pub fn new(transport: T, nodes: Vec<exocore_common::node::Node>) -> Engine<T, CP, PP> {
        // TODO: Exec Transport on runtime
        unimplemented!()
    }

    fn get_events_stream(_from_time: Instant) {
        // -> futures::Stream<Item = Event, Error = EventError> {
        unimplemented!()
    }

    fn write_entry() {
        // TODO: Send to pending store
    }

    // TODO: Sync at interval to check we didn't miss anything
    // TODO: Wait for messages from others
}

struct CommitController {}

// TODO: Stream of events ?
enum Event {
    NewPendingTransaction,
    CommitedBlock,
    FrozenBlock, // TODO: x depth
}

enum EventError {}

struct WrappedEntry {
    status: EntryStatus,
    // either from pending, or chain
}

enum EntryStatus {
    Committed,
    Pending,
}

struct EntryCondition {
    //time_condition
//block_condition
//offset_condition

// TODO: only if it's meant to be in block X at offset Y
}

struct SyncRequest {
    // TODO: A-la-ampme
// TODO: from, to, entries, heads, hash
// TODO: entries could be full or just header too (so we don't send data)
// TODO: should send full if an object has been modified by us recently and we never sent to remote
}

struct SyncRequestPayload {
    store: SyncStore,
    hash: String,
}

enum SyncStore {
    Pending,
    Chain,
}
