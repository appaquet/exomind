use tokio::prelude::*;

use crate::{Error, InMessage, OutMessage};

///
///
///
pub trait Transport: Future<Item = (), Error = Error> + Send + 'static {
    type Sink: Sink<SinkItem = OutMessage, SinkError = Error> + Send + 'static;
    type Stream: Stream<Item = InMessage, Error = Error> + Send + 'static;

    fn get_sink(&mut self) -> Self::Sink;
    fn get_stream(&mut self) -> Self::Stream;
}
