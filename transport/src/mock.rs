use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use exocore_core::cell::{LocalNode, Node, NodeId};
use exocore_core::framing::{CapnpFrameBuilder, FrameBuilder};
use exocore_core::futures::{spawn_future, Runtime};
use exocore_core::protos::generated::common_capnp::envelope;
use futures::channel::mpsc;
use futures::prelude::*;

use crate::transport::{
    ConnectionStatus, MpscHandleSink, MpscHandleStream, TransportHandleOnStart,
};
use crate::{Error, InEvent, InMessage, OutEvent, OutMessage, TransportHandle, TransportLayer};
use exocore_core::utils::handle_set::{Handle, HandleSet};
use futures::executor::block_on;
use futures::stream::Peekable;
use futures::{FutureExt, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

const CHANNELS_SIZE: usize = 1000;

type HandleKey = (NodeId, TransportLayer);

/// In memory transport used by all layers of Exocore through handles. There is
/// one handle per cell per layer.
pub struct MockTransport {
    handles_sink: Arc<Mutex<HashMap<HandleKey, HandleSink>>>,
    handle_set: HandleSet,
}

impl Default for MockTransport {
    fn default() -> MockTransport {
        MockTransport {
            handles_sink: Arc::new(Mutex::new(HashMap::new())),
            handle_set: HandleSet::new(),
        }
    }
}

impl MockTransport {
    pub fn get_transport(&self, node: LocalNode, layer: TransportLayer) -> MockTransportHandle {
        let mut handles_sink = self.handles_sink.lock().unwrap();

        let handle = self.handle_set.get_handle();

        // create channel incoming message for this node will be sent to
        let (incoming_sender, incoming_receiver) = mpsc::channel(CHANNELS_SIZE);
        handles_sink.insert(
            (node.id().clone(), layer),
            HandleSink {
                id: handle.id(),
                sender: incoming_sender,
            },
        );

        MockTransportHandle {
            handle,
            node: node.node().clone(),
            layer,
            started: false,
            handles_sink: Arc::downgrade(&self.handles_sink),
            incoming_stream: Some(incoming_receiver),
            outgoing_stream: None,
        }
    }

    pub fn notify_node_connection_status(
        &self,
        node_id: &NodeId,
        connection_status: ConnectionStatus,
    ) {
        let mut handles_sink = self.handles_sink.lock().unwrap();
        for (_handle_key, sink) in handles_sink.iter_mut() {
            let _ = sink
                .sender
                .try_send(InEvent::NodeStatus(node_id.clone(), connection_status));
        }
    }
}

/// Handle taken by a Cell layer to receive and send message for a given node
pub struct MockTransportHandle {
    handle: Handle,
    node: Node,
    layer: TransportLayer,
    started: bool,
    handles_sink: Weak<Mutex<HashMap<HandleKey, HandleSink>>>,
    incoming_stream: Option<mpsc::Receiver<InEvent>>,
    outgoing_stream: Option<mpsc::Receiver<OutEvent>>,
}

struct HandleSink {
    id: usize,
    sender: mpsc::Sender<InEvent>,
}

impl MockTransportHandle {
    pub fn into_testable(self) -> TestableTransportHandle<MockTransportHandle> {
        TestableTransportHandle::new(self)
    }
}

impl TransportHandle for MockTransportHandle {
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_started(&self) -> TransportHandleOnStart {
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
            let handles_sink_weak = Weak::clone(&self.handles_sink);
            spawn_future(async move {
                while let Some(OutEvent::Message(msg)) = outgoing_stream.next().await {
                    let handles_sink = if let Some(handles_sink) = handles_sink_weak.upgrade() {
                        handles_sink
                    } else {
                        return;
                    };
                    let mut handles_sink = handles_sink.lock().unwrap();

                    let envelope = msg.envelope_builder.as_owned_frame();
                    let in_message = InMessage::from_node_and_frame(node.clone(), envelope)
                        .expect("Couldn't get InMessage from OutMessage");
                    for dest_node in &msg.to {
                        let key = (dest_node.id().clone(), layer);
                        if let Some(node_sink) = handles_sink.get_mut(&key) {
                            let _ = node_sink
                                .sender
                                .try_send(InEvent::Message(in_message.clone()));
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
        if let Some(node_sinks) = self.handles_sink.upgrade() {
            if let Ok(mut node_sinks) = node_sinks.lock() {
                let key = (self.node.id().clone(), self.layer);

                // if another handle got registered after us, we need to keep it there
                if let Some(stream) = node_sinks.get(&key) {
                    if stream.id != self.handle.id() {
                        return;
                    }
                }

                debug!(
                    "Removing node={} layer={:?} from transport hub because it's been dropped",
                    self.node.id(),
                    self.layer,
                );
                node_sinks.remove(&key);
            }
        }
    }
}

/// Wraps a transport handle to add test methods
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
        rt.spawn(handle);
    }

    pub fn send_test_message(&mut self, rt: &mut Runtime, to: &Node, type_id: u16) {
        let mut envelope_builder = CapnpFrameBuilder::<envelope::Owned>::new();
        let mut builder = envelope_builder.get_builder();
        builder.set_type(type_id);
        builder.set_layer(TransportLayer::Chain.to_code());

        let out_message = OutMessage {
            to: vec![to.clone()],
            expiration: None,
            envelope_builder,
            connection: None,
        };

        rt.block_on(
            self.out_sink
                .as_mut()
                .unwrap()
                .send(OutEvent::Message(out_message)),
        )
        .unwrap();
    }

    pub fn receive_test_message(&mut self, rt: &mut Runtime) -> (NodeId, u16) {
        let stream = self.in_stream.as_mut().unwrap();
        let event = rt.block_on(async { stream.next().await });

        match event.unwrap() {
            InEvent::Message(message) => {
                let message_reader = message.envelope.get_reader().unwrap();
                (message.from.id().clone(), message_reader.get_type())
            }
            InEvent::NodeStatus(_, _) => self.receive_test_message(rt),
        }
    }

    pub fn receive_connection_status(&mut self, rt: &mut Runtime) -> (NodeId, ConnectionStatus) {
        let stream = self.in_stream.as_mut().unwrap();
        let event = rt.block_on(async { stream.next().await });

        match event.unwrap() {
            InEvent::NodeStatus(node_id, status) => (node_id, status),
            InEvent::Message(_) => self.receive_connection_status(rt),
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
    use exocore_core::cell::LocalNode;
    use exocore_core::futures::Runtime;

    #[test]
    fn send_and_receive() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let node1 = LocalNode::generate();

        let transport0 = hub.get_transport(node0.clone(), TransportLayer::Chain);
        let mut transport0_test = transport0.into_testable();
        transport0_test.start(&mut rt);

        let transport1 = hub.get_transport(node1.clone(), TransportLayer::Chain);
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

    #[test]
    fn connection_status_notification() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let transport0 = hub.get_transport(node0.clone(), TransportLayer::Chain);
        let mut transport0_test = transport0.into_testable();
        transport0_test.start(&mut rt);

        hub.notify_node_connection_status(node0.id(), ConnectionStatus::Connected);
        let (msg_node, status) = transport0_test.receive_connection_status(&mut rt);
        assert_eq!(&msg_node, node0.id());
        assert_eq!(status, ConnectionStatus::Connected);
    }
}
