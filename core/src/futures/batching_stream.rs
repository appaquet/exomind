use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Wraps a stream to batch all available capped number of items.
///
/// This stream doesn't block wait for a certain duration before sending
/// available items, but will consume the underlying stream until it would
/// block, or until the maximum number of items is collected.
pub struct BatchingStream<S>
where
    S: Stream + Unpin,
{
    inner: S,
    inner_done: bool,
    max_items: usize,
}

impl<S> BatchingStream<S>
where
    S: Stream + Unpin,
{
    pub fn new(inner: S, max_items: usize) -> BatchingStream<S> {
        BatchingStream {
            inner,
            inner_done: false,
            max_items,
        }
    }
}

impl<S> Stream for BatchingStream<S>
where
    S: Stream + Unpin,
{
    type Item = Vec<S::Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.inner_done {
            return Poll::Ready(None);
        }

        let max_items = self.max_items;
        let mut pinned_iner = Pin::new(&mut self.inner);
        let mut buf = Vec::new();
        for _ in 0..max_items {
            match pinned_iner.poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    buf.push(item);
                }
                Poll::Ready(None) => {
                    self.inner_done = true;
                    break;
                }
                Poll::Pending => {
                    break;
                }
            }
        }

        if !buf.is_empty() {
            Poll::Ready(Some(buf))
        } else if self.inner_done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::block_on;
    use futures::channel::mpsc;
    use futures::SinkExt;
    use tokio::stream::StreamExt;

    #[test]
    fn should_batch_items() {
        let (mut sender, receiver) = mpsc::channel(15);
        let mut batched_receiver = BatchingStream::new(receiver, 10);

        block_on(async {
            for _i in 0u8..15 {
                sender.send(()).await.unwrap();
            }
        });

        let result = block_on(async { batched_receiver.next().await });
        assert_eq!(result, Some(vec![(); 10]));

        let result = block_on(async { batched_receiver.next().await });
        assert_eq!(result, Some(vec![(); 5]));

        drop(sender);

        let result = block_on(async { batched_receiver.next().await });
        assert_eq!(result, None);
    }
}
