use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder};
use exocore_common::node::{LocalNode, Node, NodeId};
use exocore_common::protos::common_capnp::envelope;
use exocore_common::utils::futures::{spawn_future, Runtime};
use futures::channel::mpsc;
use futures::prelude::*;

use crate::transport::{MpscHandleSink, MpscHandleStream, TransportHandleOnStart};
use crate::{Error, InEvent, InMessage, OutEvent, OutMessage, TransportHandle, TransportLayer};
use exocore_common::utils::handle_set::{Handle, HandleSet};
use futures::executor::block_on;
use futures::stream::Peekable;
use futures::{FutureExt, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

const CHANNELS_SIZE: usize = 1000;

type HandleKey = (NodeId, TransportLayer);

///
/// In memory transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
///
pub struct MockTransport {
    nodes_sink: Arc<Mutex<HashMap<HandleKey, mpsc::Sender<InEvent>>>>,
    handle_set: HandleSet,
}

impl Default for MockTransport {
    fn default() -> MockTransport {
        MockTransport {
            nodes_sink: Arc::new(Mutex::new(HashMap::new())),
            handle_set: HandleSet::new(),
        }
    }
}

impl MockTransport {
    pub fn get_transport(&self, node: LocalNode, layer: TransportLayer) -> MockTransportHandle {
        let mut nodes_sink = self.nodes_sink.lock().unwrap();

        // create channel incoming message for this node will be sent to
        let (incoming_sender, incoming_receiver) = mpsc::channel(CHANNELS_SIZE);
        nodes_sink.insert((node.id().clone(), layer), incoming_sender);

        let handle = self.handle_set.get_handle();
        MockTransportHandle {
            node: node.node().clone(),
            layer,
            started: false,
            nodes_sink: Arc::downgrade(&self.nodes_sink),
            incoming_stream: Some(incoming_receiver),
            outgoing_stream: None,
            handle,
        }
    }
}

///
/// Handle taken by a Cell layer to receive and send message for a given node
///
pub struct MockTransportHandle {
    node: Node,
    layer: TransportLayer,
    started: bool,
    nodes_sink: Weak<Mutex<HashMap<HandleKey, mpsc::Sender<InEvent>>>>,
    incoming_stream: Option<mpsc::Receiver<InEvent>>,
    outgoing_stream: Option<mpsc::Receiver<OutEvent>>,
    handle: Handle,
}

impl MockTransportHandle {
    pub fn into_testable(self) -> TestableTransportHandle<MockTransportHandle> {
        TestableTransportHandle::new(self)
    }
}

impl TransportHandle for MockTransportHandle {
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_start(&self) -> TransportHandleOnStart {
        Box::new(futures::future::ready(()))
    }

    fn get_sink(&mut self) -> Self::Sink {
        let (sender, receiver) = mpsc::channel(CHANNELS_SIZE);
        self.outgoing_stream = Some(receiver);

        MpscHandleSink::new(sender)
    }

    fn get_stream(&mut self) -> Self::Stream {
        let incoming_stream = self
            .incoming_stream
            .take()
            .expect("get_stream() was already called");

        MpscHandleStream::new(incoming_stream)
    }
}

impl Future for MockTransportHandle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            info!("Transport started");

            let mut outgoing_stream = self
                .outgoing_stream
                .take()
                .expect("get_sink() didn't get called first");

            let node = self.node.clone();
            let layer = self.layer;
            let nodes_sink_weak = Weak::clone(&self.nodes_sink);
            spawn_future(async move {
                while let Some(OutEvent::Message(msg)) = outgoing_stream.next().await {
                    let nodes_sink = if let Some(nodes_sink) = nodes_sink_weak.upgrade() {
                        nodes_sink
                    } else {
                        return;
                    };
                    let mut nodes_sink = nodes_sink.lock().unwrap();

                    let envelope = msg.envelope_builder.as_owned_frame();
                    let in_message = InMessage::from_node_and_frame(node.clone(), envelope)
                        .expect("Couldn't get InMessage from OutMessage");
                    for dest_node in &msg.to {
                        let key = (dest_node.id().clone(), layer);
                        if let Some(sink) = nodes_sink.get_mut(&key) {
                            let _ = sink.try_send(InEvent::Message(in_message.clone()));
                        } else {
                            warn!(
                                "Couldn't send message to node {} since it's not in the hub anymore",
                                dest_node.id()
                            );
                        }
                    }
                }
            });

            self.started = true;
        }

        self.handle.on_set_dropped().poll_unpin(cx)
    }
}

impl Drop for MockTransportHandle {
    fn drop(&mut self) {
        if let Some(node_sinks) = self.nodes_sink.upgrade() {
            if let Ok(mut node_sinks) = node_sinks.lock() {
                debug!(
                    "Removing node {} from transport hub because it's been dropped",
                    self.node.id()
                );

                let key = (self.node.id().clone(), self.layer);
                node_sinks.remove(&key);
            }
        }
    }
}

///
/// Wraps a transport handle to add test methods
///
pub struct TestableTransportHandle<T: TransportHandle> {
    handle: Option<T>,
    out_sink: Option<T::Sink>,
    in_stream: Option<Peekable<T::Stream>>,
}

impl<T: TransportHandle> TestableTransportHandle<T> {
    pub fn new(mut handle: T) -> TestableTransportHandle<T> {
        let sink = handle.get_sink();
        let stream = handle.get_stream();

        TestableTransportHandle {
            handle: Some(handle),
            out_sink: Some(sink),
            in_stream: Some(stream.peekable()),
        }
    }

    pub fn start(&mut self, rt: &mut Runtime) {
        let handle = self.handle.take().unwrap();
        rt.spawn_std(handle);
    }

    pub fn send_test_message(&mut self, rt: &mut Runtime, to: &Node, type_id: u16) {
        let mut envelope_builder = CapnpFrameBuilder::<envelope::Owned>::new();
        let mut builder = envelope_builder.get_builder();
        builder.set_type(type_id);
        builder.set_layer(TransportLayer::Data.to_code());

        let out_message = OutMessage {
            to: vec![to.clone()],
            expiration: None,
            envelope_builder,
        };

        rt.block_on_std(
            self.out_sink
                .as_mut()
                .unwrap()
                .send(OutEvent::Message(out_message)),
        )
        .unwrap();
    }

    pub fn receive_test_message(&mut self, rt: &mut Runtime) -> (NodeId, u16) {
        let stream = self.in_stream.as_mut().unwrap();
        let event = rt.block_on_std(async { stream.next().await });

        match event.unwrap() {
            InEvent::Message(message) => {
                let message_reader = message.envelope.get_reader().unwrap();
                (message.from.id().clone(), message_reader.get_type())
            }
            InEvent::NodeStatus(_, _) => self.receive_test_message(rt),
        }
    }

    pub fn has_message(&mut self) -> Result<bool, Error> {
        block_on(async {
            let result = futures::future::poll_fn(|cx| -> Poll<bool> {
                let stream = self.in_stream.as_mut().unwrap();
                let pin_stream = Pin::new(stream);
                let res = pin_stream.poll_peek(cx).map(|res| res.is_some());

                // poll_peek blocks for next. if it's not ready, there is no message
                match res {
                    Poll::Pending => Poll::Ready(false),
                    p => p,
                }
            })
            .await;

            Ok(result)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use exocore_common::node::LocalNode;
    use exocore_common::utils::futures::Runtime;

    #[test]
    fn send_and_receive() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let node1 = LocalNode::generate();

        let transport0 = hub.get_transport(node0.clone(), TransportLayer::Data);
        let mut transport0_test = transport0.into_testable();
        transport0_test.start(&mut rt);

        let transport1 = hub.get_transport(node1.clone(), TransportLayer::Data);
        let mut transport1_test = transport1.into_testable();
        transport1_test.start(&mut rt);

        transport0_test.send_test_message(&mut rt, node1.node(), 100);

        let (msg_node, msg) = transport1_test.receive_test_message(&mut rt);
        assert_eq!(&msg_node, node0.id());
        assert_eq!(msg, 100);

        transport1_test.send_test_message(&mut rt, node0.node(), 101);

        let (msg_node, msg) = transport0_test.receive_test_message(&mut rt);
        assert_eq!(&msg_node, node1.id());
        assert_eq!(msg, 101);
    }
}
