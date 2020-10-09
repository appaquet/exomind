use futures::channel::mpsc;
use futures::channel::mpsc::SendError;
use futures::prelude::*;
use futures::sink::{Sink, SinkErrInto};
use futures::StreamExt;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{Error, InEvent, OutEvent};

/// Wraps mpsc Stream channel to map Transport's error without having a
/// convoluted type
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

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_next_unpin(cx)
    }
}

/// Wraps mpsc Sink channel to map Transport's error without having a convoluted
/// type
#[pin_project]
pub struct MpscHandleSink {
    #[pin]
    sender: SinkErrInto<mpsc::Sender<OutEvent>, OutEvent, Error>,
}

impl MpscHandleSink {
    pub fn new(sender: mpsc::Sender<OutEvent>) -> MpscHandleSink {
        MpscHandleSink {
            sender: sender.sink_err_into(),
        }
    }
}

impl Sink<OutEvent> for MpscHandleSink {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sender.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: OutEvent) -> Result<(), Self::Error> {
        self.project().sender.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sender.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sender.poll_close(cx)
    }
}

impl Into<Error> for SendError {
    fn into(self) -> Error {
        Error::Other(format!("Sink error: {}", self))
    }
}
