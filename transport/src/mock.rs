use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use futures::future;
use futures::future::FutureResult;
use futures::prelude::*;
use futures::stream::Peekable;
use futures::sync::mpsc;
use tokio::runtime::Runtime;

use exocore_common::framing::{CapnpFrameBuilder, FrameBuilder};
use exocore_common::node::{LocalNode, Node, NodeId};
use exocore_common::protos::common_capnp::envelope;
use exocore_common::tests_utils::FuturePeek;
use exocore_common::utils::completion_notifier::{CompletionListener, CompletionNotifier};

use crate::transport::{MpscHandleSink, MpscHandleStream};
use crate::{Error, InEvent, InMessage, OutEvent, OutMessage, TransportHandle, TransportLayer};

const CHANNELS_SIZE: usize = 1000;

type HandleKey = (NodeId, TransportLayer);

///
/// In memory transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
///
pub struct MockTransport {
    nodes_sink: Arc<Mutex<HashMap<HandleKey, mpsc::Sender<InEvent>>>>,
}

impl Default for MockTransport {
    fn default() -> MockTransport {
        MockTransport {
            nodes_sink: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockTransport {
    pub fn get_transport(&self, node: LocalNode, layer: TransportLayer) -> MockTransportHandle {
        let mut nodes_sink = self.nodes_sink.lock().unwrap();

        // create channel incoming message for this node will be sent to
        let (incoming_sender, incoming_receiver) = mpsc::channel(CHANNELS_SIZE);
        nodes_sink.insert((node.id().clone(), layer), incoming_sender);

        // completion handler
        let (completion_notifier, completion_listener) = CompletionNotifier::new_with_listener();

        MockTransportHandle {
            node: node.node().clone(),
            layer,
            started: false,
            nodes_sink: Arc::downgrade(&self.nodes_sink),
            incoming_stream: Some(incoming_receiver),
            outgoing_stream: None,
            completion_notifier,
            completion_listener,
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
    completion_notifier: CompletionNotifier<(), Error>,
    completion_listener: CompletionListener<(), Error>,
}

impl MockTransportHandle {
    pub fn into_testable(self) -> TestableTransportHandle<MockTransportHandle> {
        TestableTransportHandle::new(self)
    }
}

impl TransportHandle for MockTransportHandle {
    type StartFuture = FutureResult<(), Error>;
    type Sink = MpscHandleSink;
    type Stream = MpscHandleStream;

    fn on_start(&self) -> Self::StartFuture {
        futures::done(Ok(()))
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
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.started {
            info!("Transport started");

            let outgoing_stream = self
                .outgoing_stream
                .take()
                .expect("get_sink() didn't get called first");

            let node = self.node.clone();
            let layer = self.layer;
            let nodes_sink_weak = Weak::clone(&self.nodes_sink);
            let completion_handle = self.completion_notifier.clone();
            tokio::spawn(outgoing_stream.for_each(move |event| {
                match event {
                    OutEvent::Message(msg) => {
                        let nodes_sink = nodes_sink_weak.upgrade().ok_or_else(|| {
                            error!(
                                "Couldn't upgrade nodes sink, which means hub got dropped. Stopping here."
                            );
                            completion_handle
                                .complete(Err(Error::Other("Couldn't upgrade nodes sink".to_string())));
                        })?;

                        let mut nodes_sink = nodes_sink.lock().map_err(|_| {
                            error!("Couldn't get a lock on nodes sink. Stopping here.");
                            completion_handle.complete(Err(Error::Other(
                                "Couldn't get a lock on ndoes sink".to_string(),
                            )));
                        })?;

                        let envelope = msg.envelope_builder.as_owned_frame();
                        let in_message = InMessage::from_node_and_frame(node.clone(), envelope)
                            .expect("Couldn't InMessage from OutMessage");
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
                }

                Ok(())
            }));

            self.started = true;
        }

        self.completion_listener
            .poll()
            .map_err(|_err| Error::Other("Completion listener".to_string()))
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
    handle_peek: Option<FuturePeek>,
    out_sink: Option<T::Sink>,
    in_stream: Option<Peekable<T::Stream>>,
}

impl<T: TransportHandle> TestableTransportHandle<T> {
    pub fn new(mut handle: T) -> TestableTransportHandle<T> {
        let sink = handle.get_sink();
        let stream = handle.get_stream();

        TestableTransportHandle {
            handle: Some(handle),
            handle_peek: None,
            out_sink: Some(sink),
            in_stream: Some(stream.peekable()),
        }
    }

    pub fn start(&mut self, rt: &mut Runtime) {
        let handle = self.handle.take().unwrap();
        let (fut, peek) = FuturePeek::new(handle);
        self.handle_peek = Some(peek);
        rt.spawn(fut.map_err(|_| ()));
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

        let sink = rt
            .block_on(
                self.out_sink
                    .take()
                    .unwrap()
                    .send(OutEvent::Message(out_message)),
            )
            .unwrap();
        self.out_sink = Some(sink);
    }

    pub fn receive_test_message(&mut self, rt: &mut Runtime) -> (NodeId, u16) {
        let stream = self.in_stream.take().unwrap();
        let (event, stream) = rt.block_on(stream.into_future()).ok().unwrap();
        self.in_stream = Some(stream);

        match event.unwrap() {
            InEvent::Message(message) => {
                let message_reader = message.envelope.get_reader().unwrap();
                (message.from.id().clone(), message_reader.get_type())
            }
            InEvent::NodeStatus(_, _) => self.receive_test_message(rt),
        }
    }

    pub fn has_message(&mut self) -> Result<bool, Error> {
        tokio::runtime::current_thread::block_on_all(future::lazy(|| {
            let stream = self.in_stream.as_mut().unwrap();
            match stream.peek()? {
                Async::Ready(res) => Ok(res.is_some()),
                Async::NotReady => Ok(false),
            }
        }))
    }
}

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use exocore_common::node::LocalNode;
    use exocore_common::tests_utils::*;

    use super::*;

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

    #[test]
    fn completion_future() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();

        let mut transport = hub.get_transport(node0.clone(), TransportLayer::Data);
        let _transport_sink = transport.get_sink();
        let _transport_stream = transport.get_stream();

        let transport_completion_sender = transport.completion_notifier.clone();
        let (transport_future, transport_future_watch) = FuturePeek::new(transport.map_err(|_| ()));
        rt.spawn(transport_future);

        assert_eq!(transport_future_watch.get_status(), FutureStatus::NotReady);
        transport_completion_sender.complete(Result::Ok(()));
        expect_eventually(|| transport_future_watch.get_status() == FutureStatus::Ok);
    }
}
