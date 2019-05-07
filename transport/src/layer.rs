use tokio::prelude::*;

use crate::{Error, InMessage, OutMessage};

///
/// Layer of the Exocore architecture to which a message is intented / originating
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    Meta = 1,
    Common = 2,
    Data = 3,
}

impl Layer {
    pub fn from_code(code: u8) -> Option<Layer> {
        match code {
            1 => Some(Layer::Meta),
            2 => Some(Layer::Common),
            3 => Some(Layer::Data),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        self as u8
    }
}

impl Into<u8> for Layer {
    fn into(self) -> u8 {
        self.to_code()
    }
}

///
///
///
pub trait LayerStreams: Future<Item = (), Error = Error> + Send + 'static {
    type Sink: Sink<SinkItem = OutMessage, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InMessage, Error = Error> + Send + 'static;

    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}
