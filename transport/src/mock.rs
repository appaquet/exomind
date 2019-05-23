use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use futures::prelude::*;
use futures::sync::mpsc;

use exocore_common::node::{LocalNode, Node, NodeId};

use crate::transport::{MpscHandleSink, MpscHandleStream};
use crate::{Error, InMessage, OutMessage, TransportHandle};
use exocore_common::utils::completion_notifier::{CompletionListener, CompletionNotifier};
use futures::future::FutureResult;

const CHANNELS_SIZE: usize = 1000;

///
/// In memory transport used by all layers of Exocore through handles. There is one handle
/// per cell per layer.
///
pub struct MockTransport {
    nodes_sink: Arc<Mutex<HashMap<NodeId, mpsc::Sender<InMessage>>>>,
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
        let (incoming_sender, incoming_receiver) = mpsc::channel(CHANNELS_SIZE);
        nodes_sink.insert(node.id().clone(), incoming_sender);

        // completion handler
        let (completion_notifier, completion_listener) = CompletionNotifier::new_with_listener();

        MockTransportHandle {
            node: node.node().clone(),
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
    started: bool,
    nodes_sink: Weak<Mutex<HashMap<NodeId, mpsc::Sender<InMessage>>>>,
    incoming_stream: Option<mpsc::Receiver<InMessage>>,
    outgoing_stream: Option<mpsc::Receiver<OutMessage>>,
    completion_notifier: CompletionNotifier<(), Error>,
    completion_listener: CompletionListener<(), Error>,
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
            let nodes_sink_weak = Weak::clone(&self.nodes_sink);
            let completion_handle = self.completion_notifier.clone();
            tokio::spawn(outgoing_stream.for_each(move |message| {
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

                let in_message = message.to_in_message(node.clone());
                for dest_node in &message.to {
                    if let Some(sink) = nodes_sink.get_mut(dest_node.id()) {
                        let _ = sink.try_send(in_message.clone());
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
                node_sinks.remove(self.node.id());
            }
        }
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

        let transport_completion_sender = transport.completion_notifier.clone();

        let (transport_future, transport_future_watch) = FuturePeek::new(transport.map_err(|_| ()));
        rt.spawn(transport_future);

        assert_eq!(transport_future_watch.get_status(), FutureStatus::NotReady);

        transport_completion_sender.complete(Result::Ok(()));

        expect_eventually(|| transport_future_watch.get_status() == FutureStatus::Ok);
    }

    fn send_message(rt: &mut Runtime, sink: MpscHandleSink, to: Vec<Node>, type_id: u16) {
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
        stream: MpscHandleStream,
    ) -> (InMessage, MpscHandleStream) {
        let (message, stream) = rt.block_on(stream.into_future()).ok().unwrap();
        (message.unwrap(), stream)
    }
}
