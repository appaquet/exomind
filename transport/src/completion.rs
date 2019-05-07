use crate::Error;
use futures::prelude::*;
use futures::sync::oneshot;
use std::sync::{Arc, Mutex};

///
/// Exposes a barrier like structure that will resolve future once the `CompletionSender` got
/// completed.
///
#[derive(Clone)]
pub struct CompletionSender {
    sender: Arc<Mutex<Option<CompletionChannelSender>>>,
}

type CompletionChannelSender = oneshot::Sender<Result<(), Error>>;

impl CompletionSender {
    pub fn new() -> (CompletionSender, CompletionFuture) {
        let (sender, receiver) = oneshot::channel();

        let sender = CompletionSender {
            sender: Arc::new(Mutex::new(Some(sender))),
        };
        let future = CompletionFuture(receiver);
        (sender, future)
    }
}

impl CompletionSender {
    pub fn complete(&self, result: Result<(), Error>) {
        if let Ok(mut unlocked) = self.sender.lock() {
            if let Some(sender) = unlocked.take() {
                let _ = sender.send(result);
            }
        }
    }
}

pub struct CompletionFuture(oneshot::Receiver<Result<(), Error>>);

impl Future for CompletionFuture {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.0
            .poll()
            .map(|asnc| asnc.map(|_| ()))
            .map_err(|err| Error::Other(format!("Polling completion receiver failed: {:?}", err)))
    }
}
