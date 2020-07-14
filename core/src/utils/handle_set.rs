use futures::channel::{mpsc, oneshot};
use futures::future::Shared;
use futures::{Future, FutureExt, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Manages a set of handles so that their lifetime is managed along their
/// parent's lifetime.
///
/// This management happens in 3 ways:
///    1) Handle can be notified when parent is ready.
///    2) Handle can be notified when parent has been dropped.
///    3) Parent can be notified when all its handles have been dropped.
pub struct HandleSet {
    handle_dropped_sender: mpsc::Sender<()>,
    handle_dropped_receiver: mpsc::Receiver<()>,
    set_started_sender: oneshot::Sender<()>,
    set_started_receiver: Shared<oneshot::Receiver<()>>,
    set_dropped_sender: oneshot::Sender<()>,
    set_dropped_receiver: Shared<oneshot::Receiver<()>>,
    next_handle_id: Arc<AtomicUsize>,
}

impl HandleSet {
    pub fn new() -> HandleSet {
        let (handle_dropped_sender, handle_dropped_receiver) = mpsc::channel(1);
        let (set_dropped_sender, set_dropped_receiver) = oneshot::channel();
        let (set_started_sender, set_started_receiver) = oneshot::channel();

        HandleSet {
            handle_dropped_sender,
            handle_dropped_receiver,
            set_started_sender,
            set_started_receiver: set_started_receiver.shared(),
            set_dropped_sender,
            set_dropped_receiver: set_dropped_receiver.shared(),
            next_handle_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_handle(&self) -> Handle {
        let handle_id = self.next_handle_id.fetch_add(1, Ordering::SeqCst);

        Handle {
            next_handle_id: self.next_handle_id.clone(),
            handle_id,
            set_started_receiver: self.set_started_receiver.clone(),
            set_dropped_receiver: self.set_dropped_receiver.clone(),
            handle_dropped_sender: self.handle_dropped_sender.clone(),
        }
    }

    pub async fn on_handles_dropped(self) {
        let (set_started_sender, mut handle_dropped_receiver, _set_dropped_sender) = {
            let HandleSet {
                handle_dropped_sender,
                handle_dropped_receiver,
                set_started_sender,
                set_dropped_sender,
                ..
            } = self;

            drop(handle_dropped_sender);

            (
                set_started_sender,
                handle_dropped_receiver,
                set_dropped_sender,
            )
        };

        let _ = set_started_sender.send(());

        handle_dropped_receiver.next().await;
    }
}

impl Default for HandleSet {
    fn default() -> Self {
        HandleSet::new()
    }
}

pub struct Handle {
    next_handle_id: Arc<AtomicUsize>,
    handle_id: usize,
    set_started_receiver: Shared<oneshot::Receiver<()>>,
    set_dropped_receiver: Shared<oneshot::Receiver<()>>,
    handle_dropped_sender: mpsc::Sender<()>,
}

impl Handle {
    pub fn id(&self) -> usize {
        self.handle_id
    }

    pub fn set_is_started(&self) -> bool {
        self.set_started_receiver.peek().is_some()
    }

    pub fn on_set_started(&self) -> impl Future<Output = ()> {
        self.set_started_receiver.clone().map(|_| ())
    }

    pub fn on_set_dropped(&self) -> impl Future<Output = ()> {
        self.set_dropped_receiver.clone().map(|_| ())
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        let handle_id = self.next_handle_id.fetch_add(1, Ordering::SeqCst);

        Handle {
            next_handle_id: self.next_handle_id.clone(),
            handle_id,
            set_started_receiver: self.set_started_receiver.clone(),
            set_dropped_receiver: self.set_dropped_receiver.clone(),
            handle_dropped_sender: self.handle_dropped_sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::*;

    #[test]
    fn on_all_handles_dropped() -> anyhow::Result<()> {
        let rt = Runtime::new()?;
        let set = HandleSet::new();

        let handle = set.get_handle();

        let (sender, receiver) = oneshot::channel();
        rt.spawn(async move {
            set.on_handles_dropped().await;
            let _ = sender.send(());
        });

        drop(handle);

        let _ = block_on(receiver);

        Ok(())
    }

    #[test]
    fn handle_have_unique_ids() {
        let set = HandleSet::new();
        let handle1 = set.get_handle();
        let handle2 = set.get_handle();
        assert_ne!(handle1.id(), handle2.id());

        let handle3 = handle2.clone();
        assert_ne!(handle2.id(), handle3.id());
        assert_ne!(handle1.id(), handle3.id());
    }

    #[test]
    fn set_started() -> anyhow::Result<()> {
        let rt = Runtime::new()?;
        let set = HandleSet::new();

        let handle = set.get_handle();
        assert!(!handle.set_is_started());

        rt.spawn(async move {
            set.on_handles_dropped().await;
        });

        block_on(handle.on_set_started());

        assert!(handle.set_is_started());

        Ok(())
    }

    #[test]
    fn set_dropped() -> anyhow::Result<()> {
        let set = HandleSet::new();

        let handle = set.get_handle();

        drop(set);

        block_on(handle.on_set_dropped());

        Ok(())
    }
}
