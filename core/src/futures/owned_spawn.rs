use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    channel::{oneshot, oneshot::Canceled},
    prelude::*,
    FutureExt,
};

use super::spawn_future;

/// Spawns a future on current executor that can be cancelled by dropping the
/// `OwnedSpawn` handle. It is also possible to get the result of the spawned
/// future by awaiting on the handle.
pub fn owned_spawn<F, O>(fut: F) -> OwnedSpawn<O>
where
    F: Future<Output = O> + 'static + Send,
    O: Send + 'static,
{
    let (wrapped_future, spawn) = owned_future(fut);
    spawn_future(wrapped_future);
    spawn
}

/// Wraps a future that can be cancelled by dropping the `OwnedSpawn` handle.
/// It is also possible to get the result of the spawned future by awaiting on
/// the handle.
pub fn owned_future<F, O>(fut: F) -> (impl Future<Output = ()> + 'static + Send, OwnedSpawn<O>)
where
    F: Future<Output = O> + 'static + Send,
    O: Send + 'static,
{
    let (owner_drop_sender, owner_drop_receiver) = oneshot::channel();
    let (spawned_drop_sender, spawned_drop_receiver) = oneshot::channel();

    let wrapped = async move {
        let spawned_drop_sender = spawned_drop_sender;

        futures::select! {
            _ = owner_drop_receiver.fuse() => {
                // owner got dropped, doing nothing
            },
            result = fut.fuse() => {
                let _ = spawned_drop_sender.send(result);
            },
        };
    };

    let spawn = OwnedSpawn {
        _owner_drop_sender: owner_drop_sender,
        spawned_drop_receiver,
    };

    (wrapped, spawn)
}

/// Result of `owned_spawn` or `owned_future` function.
pub struct OwnedSpawn<O>
where
    O: Send + 'static,
{
    _owner_drop_sender: oneshot::Sender<()>,
    spawned_drop_receiver: oneshot::Receiver<O>,
}

impl<O> Future for OwnedSpawn<O>
where
    O: Send + 'static,
{
    type Output = Result<O, Canceled>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.spawned_drop_receiver.poll_unpin(cx)
    }
}

/// Collection of `OwnedSpawn` that allow keeping ownership over spawned futures
/// and manage their completion.
///
/// Caution: The `cleanup` method needs to be called in order to cleanup
/// completed spawns.
pub struct OwnedSpawnSet<O>
where
    O: Send + 'static,
{
    spawns: Vec<OwnedSpawn<O>>,
}

impl<O> OwnedSpawnSet<O>
where
    O: Send + 'static,
{
    pub fn new() -> OwnedSpawnSet<O> {
        OwnedSpawnSet { spawns: Vec::new() }
    }

    pub fn spawn<F>(&mut self, fut: F)
    where
        F: Future<Output = O> + 'static + Send,
    {
        let spawn = owned_spawn(fut);
        self.spawns.push(spawn);
    }

    /// Cleans up the completed spawns and return a new set with remaining
    /// spawns.
    pub async fn cleanup(self) -> OwnedSpawnSet<O> {
        let remaining_spawns = OwnedSpawnCleaner(self.spawns).await;
        OwnedSpawnSet {
            spawns: remaining_spawns,
        }
    }

    pub fn len(&self) -> usize {
        self.spawns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spawns.is_empty()
    }
}

impl<O> Default for OwnedSpawnSet<O>
where
    O: Send + 'static,
{
    fn default() -> Self {
        OwnedSpawnSet::new()
    }
}

struct OwnedSpawnCleaner<O>(Vec<OwnedSpawn<O>>)
where
    O: Send + 'static;

impl<O> Future for OwnedSpawnCleaner<O>
where
    O: Send + 'static,
{
    type Output = Vec<OwnedSpawn<O>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_empty() {
            return Poll::Ready(Vec::new());
        }

        let mut current_spawns = Vec::new();
        std::mem::swap(&mut self.0, &mut current_spawns);

        let mut remaining_spawns = Vec::new();
        for mut spawn in current_spawns {
            let polled = spawn.poll_unpin(cx);
            if polled.is_pending() {
                remaining_spawns.push(spawn);
            }
        }

        Poll::Ready(remaining_spawns)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::{super::sleep, *};

    #[tokio::test]
    async fn propagate_spawned_result() -> anyhow::Result<()> {
        let spawned = owned_spawn(async move { 1 + 1 });
        assert_eq!(2, spawned.await?);

        Ok::<(), anyhow::Error>(())
    }

    #[tokio::test]
    async fn owner_drop_cancels_spawned() -> anyhow::Result<()> {
        let dropper = Dropper::default();
        let dropped = dropper.dropped.clone();

        let spawned = owned_spawn(async move {
            sleep(Duration::from_secs(3600)).await;
            drop(dropper);
            Ok::<(), ()>(())
        });

        sleep(Duration::from_millis(100)).await;

        assert!(!dropped.load(Ordering::SeqCst));

        drop(spawned);

        sleep(Duration::from_millis(100)).await;
        assert!(dropped.load(Ordering::SeqCst));

        Ok::<(), anyhow::Error>(())
    }

    #[tokio::test]
    async fn spawn_set_cleanup() -> anyhow::Result<()> {
        let mut set = OwnedSpawnSet::<i32>::new();

        set = set.cleanup().await;

        set.spawn(async { 1 + 1 });
        assert_eq!(1, set.spawns.len());

        sleep(Duration::from_millis(100)).await;
        set = set.cleanup().await;
        assert_eq!(0, set.spawns.len());

        let dropper = Dropper::default();
        let dropped = dropper.dropped.clone();
        set.spawn(async move {
            sleep(Duration::from_secs(3600)).await;
            drop(dropper);
            1 + 1
        });

        set = set.cleanup().await;
        assert_eq!(1, set.spawns.len());

        drop(set);

        sleep(Duration::from_millis(100)).await;
        assert!(dropped.load(Ordering::SeqCst));

        Ok::<(), anyhow::Error>(())
    }

    struct Dropper {
        dropped: Arc<AtomicBool>,
    }

    impl Default for Dropper {
        fn default() -> Dropper {
            Dropper {
                dropped: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl Drop for Dropper {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst)
        }
    }
}
