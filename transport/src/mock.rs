use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use futures::prelude::*;
use futures::sync::{mpsc, oneshot};

use exocore_common::node::{LocalNode, Node, NodeId};

use crate::{Error, InMessage, OutMessage, TransportHandle};

///
/// In memory transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
///
pub struct MockTransport {
    nodes_sink: Arc<Mutex<HashMap<NodeId, mpsc::UnboundedSender<InMessage>>>>,
}

impl Default for MockTransport {
    fn default() -> MockTransport {
        MockTransport {
            nodes_sink: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockTransport {
    pub fn get_transport(&self, node: LocalNode) -> MockTransportHandle {
        let mut nodes_sink = self.nodes_sink.lock().unwrap();

        // create channel incoming message for this node will be sent to
        let (incoming_sender, incoming_receiver) = mpsc::unbounded();
        nodes_sink.insert(node.id().clone(), incoming_sender);

        // completion handler
        let (completion_sender, completion_future) = CompletionSender::new();

        MockTransportHandle {
            node: node.node().clone(),
            started: false,
            nodes_sink: Arc::downgrade(&self.nodes_sink),
            incoming_stream: Some(incoming_receiver),
            outgoing_stream: None,
            completion_sender,
            completion_future,
        }
    }
}

///
/// Handle taken by a Cell layer to receive and send message for a given node
///
pub struct MockTransportHandle {
    node: Node,
    started: bool,
    nodes_sink: Weak<Mutex<HashMap<NodeId, mpsc::UnboundedSender<InMessage>>>>,
    incoming_stream: Option<mpsc::UnboundedReceiver<InMessage>>,
    outgoing_stream: Option<mpsc::UnboundedReceiver<OutMessage>>,
    completion_sender: CompletionSender,
    completion_future: CompletionFuture,
}

impl TransportHandle for MockTransportHandle {
    type Sink = MockTransportSink;
    type Stream = MockTransportStream;

    fn get_sink(&mut self) -> Self::Sink {
        let (sender, receiver) = mpsc::unbounded();
        self.outgoing_stream = Some(receiver);

        MockTransportSink { in_channel: sender }
    }

    fn get_stream(&mut self) -> MockTransportStream {
        let incoming_stream = self
            .incoming_stream
            .take()
            .expect("get_stream() was already called");

        MockTransportStream { incoming_stream }
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
            let nodes_sink_weak = Weak::clone(&self.nodes_sink);
            let completion_handle = self.completion_sender.clone();
            tokio_executor::spawn(outgoing_stream.for_each(move |message| {
                let nodes_sink = nodes_sink_weak.upgrade().ok_or_else(|| {
                    error!(
                        "Couldn't upgrade nodes sink, which means hub got dropped. Stopping here."
                    );
                    completion_handle
                        .complete(Err(Error::Other("Couldn't upgrade nodes sink".to_string())));
                })?;

                let nodes_sink = nodes_sink.lock().map_err(|_| {
                    error!("Couldn't get a lock on nodes sink. Stopping here.");
                    completion_handle.complete(Err(Error::Other(
                        "Couldn't get a lock on ndoes sink".to_string(),
                    )));
                })?;

                let in_message = message.to_in_message(node.clone());
                for dest_node in &message.to {
                    if let Some(sink) = nodes_sink.get(dest_node.id()) {
                        let _ = sink.unbounded_send(in_message.clone());
                    } else {
                        warn!(
                            "Couldn't send message to node {} since it's not in the hub anymore",
                            dest_node.id()
                        );
                    }
                }

                Ok(())
            }));

            self.started = true;
        }

        self.completion_future.poll()
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
                node_sinks.remove(self.node.id());
            }
        }
    }
}

///
/// Wraps mpsc Stream channel to map Transport's error without having a convoluted type
///
pub struct MockTransportStream {
    incoming_stream: mpsc::UnboundedReceiver<InMessage>,
}

impl Stream for MockTransportStream {
    type Item = InMessage;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.incoming_stream.poll().map_err(|err| {
            error!(
                "Error receiving from incoming stream in MockTransportStream: {:?}",
                err
            );
            Error::Other(format!("Error receiving from incoming stream: {:?}", err))
        })
    }
}

///
/// Wraps mpsc Sink channel to map Transport's error without having a convoluted type
///
pub struct MockTransportSink {
    in_channel: mpsc::UnboundedSender<OutMessage>,
}

impl Sink for MockTransportSink {
    type SinkItem = OutMessage;
    type SinkError = Error;

    fn start_send(&mut self, item: OutMessage) -> StartSend<OutMessage, Error> {
        self.in_channel.start_send(item).map_err(|err| {
            Error::Other(format!(
                "Error calling 'start_send' to in_channel: {:?}",
                err
            ))
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Error> {
        self.in_channel.poll_complete().map_err(|err| {
            Error::Other(format!(
                "Error calling 'poll_complete' to in_channel: {:?}",
                err
            ))
        })
    }

    fn close(&mut self) -> Poll<(), Error> {
        self.in_channel
            .close()
            .map_err(|err| Error::Other(format!("Error calling 'close' to in_channel: {:?}", err)))
    }
}

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

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use exocore_common::serialization::framed::{FrameBuilder, TypedFrame};
    use exocore_common::tests_utils::*;

    use super::*;
    use exocore_common::node::LocalNode;
    use exocore_common::serialization::protos::data_transport_capnp::envelope;

    #[test]
    fn send_and_receive() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let node1 = LocalNode::generate();

        let mut transport0 = hub.get_transport(node0.clone());
        let transport0_sink = transport0.get_sink();
        let transport0_stream = transport0.get_stream();
        rt.spawn(transport0.map_err(|_| ()));

        let mut transport1 = hub.get_transport(node1.clone());
        let transport1_sink = transport1.get_sink();
        let transport1_stream = transport1.get_stream();
        rt.spawn(transport1.map_err(|_| ()));

        send_message(&mut rt, transport0_sink, vec![node1.node().clone()], 100);

        let (message, _transport1_stream) = receive_message(&mut rt, transport1_stream);
        let message_reader = message.envelope.get_typed_reader().unwrap();
        assert_eq!(message.from.id(), node0.id());
        assert_eq!(message_reader.get_type(), 100);

        send_message(&mut rt, transport1_sink, vec![node0.node().clone()], 101);

        let (message, _transport1_stream) = receive_message(&mut rt, transport0_stream);
        let message_reader = message.envelope.get_typed_reader().unwrap();
        assert_eq!(message.from.id(), node1.id());
        assert_eq!(message_reader.get_type(), 101);
    }

    #[test]
    fn completion_future() {
        let mut rt = Runtime::new().unwrap();
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();

        let mut transport = hub.get_transport(node0.clone());
        let _transport_sink = transport.get_sink();
        let _transport_stream = transport.get_stream();

        let transport_completion_sender = transport.completion_sender.clone();

        let (transport_future, transport_future_watch) =
            FutureWatch::new(transport.map_err(|_| ()));
        rt.spawn(transport_future);

        assert_eq!(transport_future_watch.get_status(), FutureStatus::NotReady);

        transport_completion_sender.complete(Result::Ok(()));

        expect_eventually(|| transport_future_watch.get_status() == FutureStatus::Ok);
    }

    fn send_message(rt: &mut Runtime, sink: MockTransportSink, to: Vec<Node>, type_id: u16) {
        let mut message = FrameBuilder::<envelope::Owned>::new();
        let mut builder = message.get_builder_typed();
        builder.set_type(type_id);

        let out_message = OutMessage {
            to,
            envelope: message,
        };

        rt.block_on(sink.send(out_message)).unwrap();
    }

    fn receive_message(
        rt: &mut Runtime,
        stream: MockTransportStream,
    ) -> (InMessage, MockTransportStream) {
        let (message, stream) = rt.block_on(stream.into_future()).ok().unwrap();
        (message.unwrap(), stream)
    }
}
