use futures::prelude::*;
use futures::sync::mpsc;

use crate::{Error, InMessage, OutMessage};
use exocore_common::node::NodeId;

///
/// Handle for a cell & layer to the transport
///
pub trait TransportHandle: Future<Item = (), Error = Error> + Send + 'static {
    type StartFuture: Future<Item = (), Error = Error> + Send + 'static;
    type Sink: Sink<SinkItem = OutEvent, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InEvent, Error = Error> + Send + 'static;

    fn on_start(&self) -> Self::StartFuture;
    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}

///
/// Layer of the Exocore architecture to which a message is intented / originating.
/// Ex: Data layer
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TransportLayer {
    None = 0,
    Meta = 1,
    Common = 2,
    Data = 3,
    Index = 4,
    Client = 5,
}

impl TransportLayer {
    pub fn from_code(code: u8) -> Option<TransportLayer> {
        match code {
            0 => Some(TransportLayer::None),
            1 => Some(TransportLayer::Meta),
            2 => Some(TransportLayer::Common),
            3 => Some(TransportLayer::Data),
            4 => Some(TransportLayer::Index),
            5 => Some(TransportLayer::Client),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        self as u8
    }
}

///
/// Connection status of a remote node via the transport.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Clone)]
pub enum InEvent {
    Message(Box<InMessage>),
    NodeStatus(NodeId, ConnectionStatus),
}

pub enum OutEvent {
    Message(OutMessage),
}

///
/// Wraps mpsc Stream channel to map Transport's error without having a convoluted type
///
pub struct MpscHandleStream {
    receiver: mpsc::Receiver<InEvent>,
}

impl MpscHandleStream {
    pub fn new(receiver: mpsc::Receiver<InEvent>) -> MpscHandleStream {
        MpscHandleStream { receiver }
    }
}

impl Stream for MpscHandleStream {
    type Item = InEvent;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.receiver.poll().map_err(|_err| {
            error!("Error receiving from incoming stream in MockTransportStream",);
            Error::Other("Error receiving from incoming stream".to_string())
        })
    }
}

///
/// Wraps mpsc Sink channel to map Transport's error without having a convoluted type
///
pub struct MpscHandleSink {
    sender: mpsc::Sender<OutEvent>,
}

impl MpscHandleSink {
    pub fn new(sender: mpsc::Sender<OutEvent>) -> MpscHandleSink {
        MpscHandleSink { sender }
    }
}

impl Sink for MpscHandleSink {
    type SinkItem = OutEvent;
    type SinkError = Error;

    fn start_send(&mut self, item: OutEvent) -> StartSend<OutEvent, Error> {
        self.sender.start_send(item).map_err(|err| {
            Error::Other(format!("Error calling 'start_send' to in_channel: {}", err))
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Error> {
        self.sender.poll_complete().map_err(|err| {
            Error::Other(format!(
                "Error calling 'poll_complete' to in_channel: {}",
                err
            ))
        })
    }

    fn close(&mut self) -> Poll<(), Error> {
        self.sender
            .close()
            .map_err(|err| Error::Other(format!("Error calling 'close' to in_channel: {}", err)))
    }
}
