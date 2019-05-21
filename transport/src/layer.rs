use futures::prelude::*;

use crate::{Error, InMessage, OutMessage};

/// Layer of the Exocore architecture to which a message is intented / originating.
/// Ex: Data layer
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TransportLayer {
    Meta = 1,
    Common = 2,
    Data = 3,
}

impl TransportLayer {
    pub fn from_code(code: u8) -> Option<TransportLayer> {
        match code {
            1 => Some(TransportLayer::Meta),
            2 => Some(TransportLayer::Common),
            3 => Some(TransportLayer::Data),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        self as u8
    }
}

impl Into<u8> for TransportLayer {
    fn into(self) -> u8 {
        self.to_code()
    }
}

/// Handle to the Transport that allows a layer of the architecture to communicate with
/// other nodes for a given cell.
pub trait TransportHandle: Future<Item = (), Error = Error> + Send + 'static {
    type Sink: Sink<SinkItem = OutMessage, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InMessage, Error = Error> + Send + 'static;

    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}
