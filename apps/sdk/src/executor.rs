//! Simple executor to be used inside of an application runtime. The executor is
//! polled when needed by the runtime.
//!
//! This is partially copied and adapted from https://rust-lang.github.io/async-book/02_execution/04_executor.html.

use std::{
    future::Future,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::Context,
};

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};

const MAX_QUEUED_TASKS: usize = 100_000;

/// Spawns a task onto the executor.
pub fn spawn(future: impl Future<Output = ()> + 'static + Send) {
    EXECUTOR.spawner.spawn(future);
}

pub(crate) fn poll_executor() {
    let executor = EXECUTOR.executor.lock().unwrap();
    executor.poll();
}

lazy_static! {
    static ref EXECUTOR: ExecutorPair = init();
}

struct ExecutorPair {
    executor: Mutex<Executor>,
    spawner: Spawner,
}

fn init() -> ExecutorPair {
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    let executor = Executor { ready_queue };
    let spawner = Spawner { task_sender };

    ExecutorPair {
        executor: Mutex::new(executor),
        spawner,
    }
}

/// Task executor that receives tasks off of a channel and runs them.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    fn poll(&self) {
        while let Ok(task) = self.ready_queue.try_recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if future.as_mut().poll(context).is_pending() {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::oneshot;

    use super::*;

    #[test]
    fn simple_two_tasks_channels() {
        let (sender1, mut receiver1) = oneshot::channel();
        spawn(async move {
            sender1.send("hello").unwrap();
        });

        // nothing has been executed yet, so should not have received on first channel
        assert!(receiver1.try_recv().unwrap().is_none());

        // create second task which will receive from first channel, then send to
        // another one
        let (sender2, mut receiver2) = oneshot::channel();
        spawn(async move {
            sender2.send(receiver1.await.unwrap()).unwrap();
        });

        // nothing has been executed yet, so should not have received on second channel
        assert!(receiver2.try_recv().unwrap().is_none());

        // poll executor, should have received from first channel and forward to second
        poll_executor();

        // second channel should have received
        assert_eq!(receiver2.try_recv().unwrap(), Some("hello"));
    }
}
