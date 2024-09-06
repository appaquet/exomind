use std::task::Poll;

use bytes::Bytes;
use futures::{AsyncRead, AsyncWrite, Stream};
use pin_project::pin_project;

// Creates a channel of bytes that implements AsyncRead and AsyncWrite.
// This can be used to send and receive data between two endpoints.
pub fn new(buf_size: usize) -> (BytesChannelSender, BytesChannelReceiver) {
    let (sender, receiver) = futures::channel::mpsc::channel(buf_size);
    (
        BytesChannelSender { sender },
        BytesChannelReceiver {
            receiver,
            reserve: None,
        },
    )
}

#[pin_project]
pub struct BytesChannelSender {
    #[pin]
    sender: futures::channel::mpsc::Sender<Bytes>,
}

impl AsyncWrite for BytesChannelSender {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();

        // check if channel is ready to receive new data
        match this.sender.poll_ready(cx) {
            Poll::Ready(Ok(_)) => {}
            Poll::Ready(Err(err)) => {
                return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)))
            }
            Poll::Pending => return Poll::Pending,
        }

        // channel is ready, send it
        let bytes = Bytes::copy_from_slice(buf);
        if let Err(err) = this.sender.start_send(bytes) {
            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)));
        }

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
pub struct BytesChannelReceiver {
    #[pin]
    receiver: futures::channel::mpsc::Receiver<Bytes>,
    reserve: Option<Bytes>,
}

impl AsyncRead for BytesChannelReceiver {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.project();

        let mut bytes = if let Some(reserve) = this.reserve.take() {
            reserve
        } else {
            match this.receiver.poll_next(cx) {
                Poll::Ready(Some(bytes)) => bytes,
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        };

        if bytes.len() > buf.len() {
            buf.copy_from_slice(&bytes[..buf.len()]);
            *this.reserve = Some(bytes.split_off(buf.len()));
            Poll::Ready(Ok(buf.len()))
        } else {
            let rest = bytes.len();
            buf[..rest].copy_from_slice(&bytes);
            Poll::Ready(Ok(rest))
        }
    }
}

#[cfg(test)]
mod tests {
    use exocore_core::futures::spawn_future;
    use futures::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bytes_channel() {
        let (mut sender, mut receiver) = new(3);

        // send data in chunks of random size
        let tx_sizes = [1, 11, 23, 46, 31, 67, 112, 23, 41];
        spawn_future(async move {
            for size in tx_sizes {
                let mut data = Vec::new();
                for _ in 0..size {
                    data.push(size);
                }
                let _ = sender.write(&data).await.unwrap();
            }
        });

        // receive data in chunks of random size
        let mut received = Vec::new();
        let rx_sizes = [
            31, 1, 52, 143, 3, 113, 23, 98, 15, 312, 31, 321, 2, 41, 33, 2, 44, 11, 134,
        ];
        for size in rx_sizes {
            let mut buf = vec![0; size];
            let read = receiver.read(&mut buf).await.unwrap();
            if read == 0 {
                break;
            }

            received.extend_from_slice(&buf[..read]);
        }

        // expect received data to be equal to sent data
        let expected_size = tx_sizes.iter().map(|i| usize::from(*i)).sum::<usize>();
        assert_eq!(received.len(), expected_size);
    }
}
