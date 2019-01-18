use crate::chain;
use crate::pending;
use crate::transport;

use exocore_common;

use futures::sync::mpsc;
use tokio;
use tokio::prelude::*;
use tokio::timer::Interval;

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// TODO: Should have a "EngineState" so that we can easily test the states transition / actions
// TODO: If node has access to data, it needs ot check its integrity by the upper layer
// TODO: If not, a node needs to wait for a majority of nodes that has data
// TODO: should send full if an object has been modified by us recently and we never sent to remote

const ENGINE_MANAGE_TIMER_INTERVAL: Duration = Duration::from_secs(1);

pub struct Engine<T, CP, PP>
where
    T: transport::Transport,
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    started: bool,
    transport: Option<T>,
    inner: Arc<RwLock<Inner<CP, PP>>>,
}

struct Inner<CP, PP>
where
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    nodes: Vec<exocore_common::node::Node>,
    pending: pending::Store<PP>,
    chain: chain::Chain<CP>,
    last_error: Option<Error>,
}

impl<T, CP, PP> Engine<T, CP, PP>
where
    T: transport::Transport,
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    pub fn new(
        transport: T,
        chain_persistence: CP,
        pending_persistence: PP,
        nodes: Vec<exocore_common::node::Node>,
    ) -> Engine<T, CP, PP> {
        let pending = pending::Store::new(pending_persistence);
        let context = Arc::new(RwLock::new(Inner {
            nodes,
            pending,
            chain: chain::Chain::new(chain_persistence),
            last_error: None,
        }));

        Engine {
            started: false,
            inner: context,
            transport: Some(transport),
        }
    }

    pub fn get_handle(&mut self) -> Handle {
        // TODO:
        unimplemented!()
    }

    fn start(&mut self) -> Result<(), Error> {
        let (transport_out_sink, transport_in_stream) =
            self.transport.take().ok_or(Error::Unknown)?.split();

        // handle messages going to transport
        let (transport_send, transport_receiver) = mpsc::unbounded();
        tokio::spawn(
            transport_receiver
                .map_err(|err| transport::Error::Unknown)
                .forward(transport_out_sink)
                .map(|_| ())
                .map_err(|err| {
                    // TODO: Mark engine failed
                    error!("Error forwarding to transport sink: {:?}", err);
                }),
        );

        // handle transport's incoming messages
        tokio::spawn(
            transport_in_stream
                .for_each(|msg| {
                    info!("Got incoming message");

                    Ok(())
                })
                .map_err(|err| {
                    // TODO: Mark engine failed
                    error!("Error handling incoming message from transport: {:?}", err);
                }),
        );

        tokio::spawn({
            Interval::new_interval(ENGINE_MANAGE_TIMER_INTERVAL)
                .for_each(|_| {
                    // TODO: Sync at interval to check we didn't miss anything
                    // TODO: Maybe propose a new block

                    Ok(())
                })
                .map_err(|err| {
                    // TODO: Mark engine failed
                    error!("Error in management timer: {:?}", err);
                })
        });

        self.started = true;

        Ok(())
    }
}

impl<T, CP, PP> Future for Engine<T, CP, PP>
where
    T: transport::Transport,
    CP: chain::Persistence,
    PP: pending::Persistence,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<<Self as Future>::Item>, <Self as Future>::Error> {
        if !self.started {
            self.start()?;
        }

        // TODO: Check if failed

        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Error {
    Unknown,
}

pub struct Handle {}

impl Handle {
    pub fn write_entry(&self) {
        // TODO: Send to pending store
    }

    pub fn get_events_stream(&self, _from_time: Instant)
    /*-> impl futures::Stream<Item = Event, Error = Error>*/
    {
        unimplemented!()
    }
}

///
///
///
struct CommitController {}

enum Event {
    NewPendingTransaction,
    CommitedBlock,
    FrozenBlock, // TODO: x depth
}

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
