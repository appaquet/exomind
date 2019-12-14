use futures01::prelude::*;
use futures01::sync::oneshot;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, Weak};

///
/// Allow notifying listeners on completion or drop of the completer.
///
#[derive(Clone)]
pub struct CompletionNotifier<S: Clone, E: Clone + Debug + Send + Sync + 'static> {
    inner: Arc<RwLock<Inner<S, E>>>,
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> CompletionNotifier<S, E> {
    pub fn new() -> CompletionNotifier<S, E> {
        let inner = Inner {
            result: None,
            next: 0,
            senders: HashMap::new(),
        };

        CompletionNotifier {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn new_with_listener() -> (CompletionNotifier<S, E>, CompletionListener<S, E>) {
        let notifier = Self::new();
        let listener = notifier
            .get_listener()
            .map_err(|_| ())
            .expect("Couldn't get listener");
        (notifier, listener)
    }

    pub fn complete(&self, result: Result<S, E>) {
        if let Ok(mut inner) = self.inner.write() {
            inner.try_complete(result.map_err(CompletionError::UserError));
        }
    }

    pub fn get_listener(&self) -> Result<CompletionListener<S, E>, CompletionError<E>> {
        let mut inner = self.inner.write().map_err(|_| CompletionError::Dropped)?;

        let (id, receiver) = inner.new_receiver();
        Ok(CompletionListener {
            id,
            inner: Arc::downgrade(&self.inner),
            receiver,
        })
    }

    pub fn is_complete(&self) -> bool {
        if let Ok(inner) = self.inner.read() {
            inner.result.is_some()
        } else {
            true
        }
    }

    pub fn result(&self) -> Option<Result<S, CompletionError<E>>> {
        if let Ok(inner) = self.inner.read() {
            inner.result.clone()
        } else {
            Some(Err(CompletionError::Dropped))
        }
    }
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> Default for CompletionNotifier<S, E> {
    fn default() -> Self {
        CompletionNotifier::new()
    }
}

struct Inner<S: Clone, E: Clone + Debug + Send + Sync + 'static> {
    result: Option<Result<S, CompletionError<E>>>,
    next: usize,
    senders: HashMap<usize, oneshot::Sender<Result<S, CompletionError<E>>>>,
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> Inner<S, E> {
    fn new_receiver(&mut self) -> (usize, oneshot::Receiver<Result<S, CompletionError<E>>>) {
        let id = self.next;
        self.next += 1;

        let (sender, receiver) = oneshot::channel();
        self.senders.insert(id, sender);

        (id, receiver)
    }

    fn try_complete(&mut self, result: Result<S, CompletionError<E>>) {
        if self.result.is_none() {
            self.result = Some(result.clone());

            for (_, sender) in self.senders.drain() {
                let _ = sender.send(result.clone());
            }
        }
    }
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> Drop for Inner<S, E> {
    fn drop(&mut self) {
        self.try_complete(Err(CompletionError::Dropped));
    }
}

///
/// Exposes a future that will resolve when the notifier has completed or has been dropped.
///
pub struct CompletionListener<S: Clone, E: Clone + Debug + Send + Sync + 'static> {
    id: usize,
    inner: Weak<RwLock<Inner<S, E>>>,
    receiver: oneshot::Receiver<Result<S, CompletionError<E>>>,
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> CompletionListener<S, E> {
    pub fn result(&self) -> Option<Result<S, CompletionError<E>>> {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(inner) = inner.read() {
                inner.result.clone()
            } else {
                Some(Err(CompletionError::Dropped))
            }
        } else {
            Some(Err(CompletionError::Dropped))
        }
    }

    pub fn try_clone(&self) -> Result<CompletionListener<S, E>, CompletionError<E>> {
        let inner = self.inner.upgrade().ok_or(CompletionError::Dropped)?;
        let mut inner = inner.write().map_err(|_| CompletionError::Dropped)?;

        let (id, receiver) = inner.new_receiver();
        Ok(CompletionListener {
            id,
            inner: Weak::clone(&self.inner),
            receiver,
        })
    }
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> Future for CompletionListener<S, E> {
    type Item = S;
    type Error = CompletionError<E>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        {
            let inner = self.inner.upgrade().ok_or(CompletionError::Dropped)?;
            let inner = inner.read().map_err(|_| CompletionError::Dropped)?;

            if let Some(result) = &inner.result {
                return result.clone().map(Async::Ready);
            }
        }

        self.receiver
            .poll()
            .map_err(|_| CompletionError::Dropped)
            .and_then(|asc| match asc {
                Async::NotReady => Ok(Async::NotReady),
                Async::Ready(Ok(res)) => Ok(Async::Ready(res)),
                Async::Ready(Err(err)) => Err(err),
            })
    }
}

impl<S: Clone, E: Clone + Debug + Send + Sync + 'static> Drop for CompletionListener<S, E> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.senders.remove(&self.id);
            }
        }
    }
}

#[derive(Clone, Debug, Fail)]
pub enum CompletionError<E: Clone + Debug + Send + Sync + 'static> {
    #[fail(display = "User error: {:?}", _0)]
    UserError(E),
    #[fail(display = "Completer notifier got dropped")]
    Dropped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_utils::{expect_eventually, FuturePeek, FutureStatus};

    #[test]
    fn clone_and_drop_listener() -> Result<(), failure::Error> {
        let (completion, listener1) = CompletionNotifier::<bool, ()>::new_with_listener();
        let listener2 = completion.get_listener().unwrap();
        let listener3 = listener1.try_clone().unwrap();

        {
            let inner = completion.inner.read().unwrap();
            assert_eq!(3, inner.senders.len());
        }

        {
            drop(listener1);
            let inner = completion.inner.read().unwrap();
            assert_eq!(2, inner.senders.len());
        }

        {
            drop(listener2);
            let inner = completion.inner.read().unwrap();
            assert_eq!(1, inner.senders.len());
        }

        {
            drop(listener3);
            let inner = completion.inner.read().unwrap();
            assert_eq!(0, inner.senders.len());
        }

        Ok(())
    }

    #[test]
    fn drop_completion_notifies() -> Result<(), failure::Error> {
        let mut rt = tokio::runtime::Runtime::new()?;

        let (completion, listener) = CompletionNotifier::<bool, ()>::new_with_listener();

        let (wrapped_future, future_watcher) = FuturePeek::new(Box::new(listener));
        rt.spawn(wrapped_future.map(|_| ()).map_err(|_| ()));

        assert_eq!(FutureStatus::NotReady, future_watcher.get_status());
        drop(completion);
        expect_eventually(|| FutureStatus::Failed == future_watcher.get_status());

        Ok(())
    }

    #[test]
    fn complete_notifies_async() -> Result<(), failure::Error> {
        let mut rt = tokio::runtime::Runtime::new()?;

        let (completion, listener) = CompletionNotifier::<bool, ()>::new_with_listener();
        completion.complete(Ok(true));

        let (wrapped_future, future_watcher) = FuturePeek::new(Box::new(listener));
        rt.spawn(wrapped_future.map(|_| ()).map_err(|_| ()));

        assert_eq!(FutureStatus::NotReady, future_watcher.get_status());
        completion.complete(Ok(true));
        expect_eventually(|| FutureStatus::Ok == future_watcher.get_status());

        Ok(())
    }

    #[test]
    fn already_complete_notifies() -> Result<(), failure::Error> {
        let mut rt = tokio::runtime::Runtime::new()?;

        let (completion, listener) = CompletionNotifier::<bool, ()>::new_with_listener();
        completion.complete(Ok(true));
        assert_eq!(Some(true), listener.result().and_then(Result::ok));
        assert_eq!(Some(true), rt.block_on(listener).ok());

        Ok(())
    }
}
