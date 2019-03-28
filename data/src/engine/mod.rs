use std;
use std::sync::{Arc, RwLock, Weak};
use std::time::{Duration, Instant};

use futures::prelude::*;
use futures::sync::mpsc;
use futures::sync::oneshot;
use tokio;
use tokio::timer::Interval;

use exocore_common;
use exocore_common::node::Node;

use crate::chain;
use crate::pending;
use crate::transport;

mod pending_sync;

const ENGINE_MANAGE_TIMER_INTERVAL: Duration = Duration::from_secs(1);

///
///
///
pub struct Engine<T, CS, PS>
where
    T: transport::Transport,
    CS: chain::Store,
    PS: pending::Store,
{
    started: bool,
    transport: T,
    inner: Arc<RwLock<Inner<CS, PS>>>,
    completion_receiver: oneshot::Receiver<Result<(), Error>>,
}

impl<T, CS, PS> Engine<T, CS, PS>
where
    T: transport::Transport,
    CS: chain::Store,
    PS: pending::Store,
{
    pub fn new(
        transport: T,
        chain_store: CS,
        pending_store: PS,
        nodes: Vec<exocore_common::node::Node>,
    ) -> Engine<T, CS, PS> {
        let (completion_sender, completion_receiver) = oneshot::channel();

        let context = Arc::new(RwLock::new(Inner {
            nodes,
            pending_store,
            pending_syncer: pending_sync::Synchronizer::new(),
            chain_store,
            transport_sender: None,
            completion_sender: Some(completion_sender),
        }));

        Engine {
            started: false,
            inner: context,
            transport,
            completion_receiver,
        }
    }

    pub fn get_handle(&mut self) -> Handle<CS, PS> {
        // TODO: Add a new channel to which we should send streams events. It should probably be bounded & having an error to indicate that we could't push

        Handle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let transport_in_stream = self.transport.get_stream();
        let transport_out_sink = self.transport.get_sink();

        // create channel to send messages to transport's sink
        {
            let weak_inner = Arc::downgrade(&self.inner);
            let (transport_out_channel_sender, transport_out_channel_receiver) = mpsc::unbounded();
            tokio::spawn(
                transport_out_channel_receiver
                    .map_err(|err| {
                        transport::Error::Other(format!(
                            "Couldn't send to transport_out channel's receiver: {:?}",
                            err
                        ))
                    })
                    .forward(transport_out_sink)
                    .map(|_| ())
                    .map_err(move |err| {
                        Self::handle_spawned_future_error(
                            "transport incoming stream",
                            &weak_inner,
                            Error::Transport(err),
                        )
                    }),
            );

            let mut unlocked_inner = self.inner.write()?;
            unlocked_inner.transport_sender = Some(transport_out_channel_sender);
        }

        // handle transport's incoming messages
        {
            let weak_inner1 = Arc::downgrade(&self.inner);
            let weak_inner2 = Arc::downgrade(&self.inner);
            tokio::spawn(
                transport_in_stream
                    .map_err(Error::Transport)
                    .for_each(move |msg| Self::handle_incoming_message(&weak_inner1, msg))
                    .map_err(move |err| {
                        Self::handle_spawned_future_error(
                            "transport incoming stream",
                            &weak_inner2,
                            err,
                        )
                    }),
            );
        }

        // management timer
        {
            let weak_inner1 = Arc::downgrade(&self.inner);
            let weak_inner2 = Arc::downgrade(&self.inner);
            tokio::spawn({
                Interval::new_interval(ENGINE_MANAGE_TIMER_INTERVAL)
                    .map_err(|err| Error::Other(format!("Interval error: {:?}", err)))
                    .for_each(move |_| Self::handle_management_timer_tick(&weak_inner1))
                    .map_err(move |err| {
                        Self::handle_spawned_future_error("management timer", &weak_inner2, err)
                    })
            });
        }

        self.started = true;
        Ok(())
    }

    fn handle_incoming_message(
        _inner: &Weak<RwLock<Inner<CS, PS>>>,
        _msg: transport::InMessage,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn handle_management_timer_tick(_inner: &Weak<RwLock<Inner<CS, PS>>>) -> Result<(), Error> {
        // TODO: Sync at interval to check we didn't miss anything
        // TODO: Maybe propose a new block

        unimplemented!()
    }

    fn handle_spawned_future_error(
        future_name: &str,
        weak_inner: &Weak<RwLock<Inner<CS, PS>>>,
        error: Error,
    ) {
        error!("Got an error in future {}: {:?}", future_name, error);

        let locked_inner = if let Some(locked_inner) = weak_inner.upgrade() {
            locked_inner
        } else {
            return;
        };

        let mut inner = if let Ok(inner) = locked_inner.write() {
            inner
        } else {
            return;
        };

        inner.try_complete(Err(Error::Other(format!(
            "Couldn't send to completion channel: {:?}",
            error
        ))));
    }
}

impl<T, CS, PS> Future for Engine<T, CS, PS>
where
    T: transport::Transport,
    CS: chain::Store,
    PS: pending::Store,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<()>, Error> {
        if !self.started {
            self.start()?;
        }

        // check if engine was stopped or failed
        let _ = try_ready!(self
            .completion_receiver
            .poll()
            .map_err(|_cancel| Error::Other("Completion receiver has been cancelled".to_string())));

        Ok(Async::Ready(()))
    }
}

///
///
///
struct Inner<CS, PS>
where
    CS: chain::Store,
    PS: pending::Store,
{
    nodes: Vec<Node>,
    pending_store: PS,
    pending_syncer: pending_sync::Synchronizer<PS>,
    chain_store: CS,
    transport_sender: Option<mpsc::UnboundedSender<transport::OutMessage>>,
    completion_sender: Option<oneshot::Sender<Result<(), Error>>>,
}

impl<CS, PS> Inner<CS, PS>
where
    CS: chain::Store,
    PS: pending::Store,
{
    fn try_complete(&mut self, result: Result<(), Error>) {
        if let Some(sender) = self.completion_sender.take() {
            let _ = sender.send(result);
        }
    }
}

///
/// Handle ot the Engine, allowing communication with the engine.
/// The engine itself is owned by an future executor.
///
pub struct Handle<CS, PS>
where
    CS: chain::Store,
    PS: pending::Store,
{
    inner: Weak<RwLock<Inner<CS, PS>>>,
}

impl<CS, PS> Handle<CS, PS>
where
    CS: chain::Store,
    PS: pending::Store,
{
    pub fn write_entry(&self) -> Result<(), Error> {
        // TODO: Write to pending store
        // TODO: Force sync

        unimplemented!()
    }

    pub fn get_events_stream(
        &self,
        _from_time: Instant,
    ) -> Box<dyn futures::Stream<Item = Event, Error = Error>> {
        unimplemented!()
    }
}

///
///
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Try to lock a mutex that was poisoned")]
    Poisoned,
    #[fail(display = "Error in transport: {:?}", _0)]
    Transport(transport::Error),
    #[fail(display = "Inner was dropped or couldn't get locked")]
    InnerUpgrade,
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

///
///
///
pub struct CommitController {}

pub enum Event {
    NewPendingTransaction,
    CommittedBlock,
    FrozenBlock, // TODO: x depth
}

pub struct WrappedEntry {
    pub status: EntryStatus,
    // either from pending, or chain
}

pub enum EntryStatus {
    Committed,
    Pending,
}

#[cfg(test)]
mod tests {
    #[test]
    fn engine_completion_on_error() {}
}
