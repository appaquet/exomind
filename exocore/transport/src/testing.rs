use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
    task::{Context, Poll},
};

use exocore_core::{
    cell::{Cell, LocalNode, Node, NodeId},
    framing::CapnpFrameBuilder,
    futures::{owned_spawn, spawn_future, OwnedSpawn},
    utils::handle_set::{Handle, HandleSet},
};
use exocore_protos::generated::data_chain_capnp::block_operation_header;
use futures::{channel::mpsc, prelude::*, stream::Peekable, FutureExt, StreamExt};

use crate::{
    streams::{MpscHandleSink, MpscHandleStream},
    transport::{ConnectionStatus, TransportHandleOnStart},
    Error, InEvent, InMessage, OutEvent, OutMessage, ServiceType, TransportServiceHandle,
};

const CHANNELS_SIZE: usize = 1000;

type HandleKey = (NodeId, ServiceType);

/// In memory transport used by all services of Exocore through handles. There
/// is one handle per cell per service.
pub struct MockTransport {
    service_sinks: Arc<Mutex<HashMap<HandleKey, ServiceSink>>>,
    handle_set: HandleSet,
}

impl Default for MockTransport {
    fn default() -> MockTransport {
        MockTransport {
            service_sinks: Arc::new(Mutex::new(HashMap::new())),
            handle_set: HandleSet::new(),
        }
    }
}

impl MockTransport {
    pub fn get_transport(
        &self,
        node: LocalNode,
        service_type: ServiceType,
    ) -> MockTransportServiceHandle {
        let mut service_sinks = self.service_sinks.lock().unwrap();

        let handle = self.handle_set.get_handle();

        // create channel incoming message for this node will be sent to
        let (incoming_sender, incoming_receiver) = mpsc::channel(CHANNELS_SIZE);
        service_sinks.insert(
            (node.id().clone(), service_type),
            ServiceSink {
                id: handle.id(),
                sender: incoming_sender,
            },
        );

        MockTransportServiceHandle {
            handle,
            node: node.node().clone(),
            service_type,
            started: false,
            service_sinks: Arc::downgrade(&self.service_sinks),
            incoming_stream: Some(incoming_receiver),
            outgoing_stream: None,
        }
    }

    pub fn notify_node_connection_status(
        &self,
        node_id: &NodeId,
        connection_status: ConnectionStatus,
    ) {
        let mut handles_sink = self.service_sinks.lock().unwrap();
        for (_handle_key, sink) in handles_sink.iter_mut() {
            let _ = sink
                .sender
                .try_send(InEvent::NodeStatus(node_id.clone(), connection_status));
        }
    }
}

/// Handle taken by a Cell service to receive and send message for a given node
pub struct MockTransportServiceHandle {
    handle: Handle,
    node: Node,
    service_type: ServiceType,
    started: bool,
    service_sinks: Weak<Mutex<HashMap<HandleKey, ServiceSink>>>,
    incoming_stream: Option<mpsc::Receiver<InEvent>>,
    outgoing_stream: Option<mpsc::Receiver<OutEvent>>,
}

struct ServiceSink {
    id: usize,
    sender: mpsc::Sender<InEvent>,
}

impl TransportServiceHandle for MockTransportServiceHandle {
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

impl Future for MockTransportServiceHandle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            info!("Transport started");

            let mut outgoing_stream = self
                .outgoing_stream
                .take()
                .expect("get_sink() didn't get called first");

            let node = self.node.clone();
            let service_type = self.service_type;
            let handles_sink_weak = Weak::clone(&self.service_sinks);
            spawn_future(async move {
                while let Some(OutEvent::Message(msg)) = outgoing_stream.next().await {
                    let Some(handles_sink) = handles_sink_weak.upgrade() else {
                        return;
                    };
                    let mut handles_sink = handles_sink.lock().unwrap();

                    let in_message = msg
                        .to_in_message(node.clone())
                        .expect("Couldn't get InMessage from OutMessage");
                    let dest_node = msg
                        .destination
                        .expect("Message didn't have a destination node");
                    let key = (dest_node.id().clone(), service_type);
                    if let Some(node_sink) = handles_sink.get_mut(&key) {
                        let _ = node_sink.sender.try_send(InEvent::Message(in_message));
                    } else {
                        warn!(
                            "Couldn't send message to node {} since it's not in the hub anymore",
                            dest_node.id()
                        );
                    }
                }
            });

            self.started = true;
        }

        self.handle.on_set_dropped().poll_unpin(cx)
    }
}

impl Drop for MockTransportServiceHandle {
    fn drop(&mut self) {
        if let Some(node_sinks) = self.service_sinks.upgrade() {
            if let Ok(mut node_sinks) = node_sinks.lock() {
                let key = (self.node.id().clone(), self.service_type);

                // if another handle got registered after us, we need to keep it there
                if let Some(stream) = node_sinks.get(&key) {
                    if stream.id != self.handle.id() {
                        return;
                    }
                }

                debug!(
                    "Removing node={} service_type={:?} from transport hub because it's been dropped",
                    self.node.id(),
                    self.service_type,
                );
                node_sinks.remove(&key);
            }
        }
    }
}

impl MockTransportServiceHandle {
    pub fn into_testable_handle(self, cell: Cell) -> TestableTransportHandle {
        TestableTransportHandle::new(self, cell)
    }
}

/// Wraps a transport handle to add test methods
pub struct TestableTransportHandle {
    cell: Cell,
    out_sink: mpsc::UnboundedSender<OutEvent>,
    in_stream: Peekable<mpsc::UnboundedReceiver<InEvent>>,
    received_events: Arc<AtomicUsize>,
    node_status: Arc<futures::lock::Mutex<HashMap<NodeId, ConnectionStatus>>>,
    _in_spawn: OwnedSpawn<()>,
    _out_spawn: OwnedSpawn<()>,
}

impl TestableTransportHandle {
    pub fn new<T: TransportServiceHandle>(mut handle: T, cell: Cell) -> TestableTransportHandle {
        let mut handle_sink = handle.get_sink();
        let mut handle_stream = handle.get_stream();
        spawn_future(handle);

        let received_events = Arc::new(AtomicUsize::new(0));
        let node_status = Arc::new(futures::lock::Mutex::new(HashMap::new()));

        let (in_stream, in_spawn) = {
            let received_events = received_events.clone();
            let node_status = node_status.clone();

            let (mut in_sender, in_receiver) = mpsc::unbounded();
            let spawn = owned_spawn(async move {
                while let Some(event) = handle_stream.next().await {
                    received_events.fetch_add(1, Ordering::Relaxed);

                    if matches!(&event, InEvent::NodeStatus(_, _)) {
                        let mut node_status = node_status.lock().await;
                        if let InEvent::NodeStatus(node_id, status) = &event {
                            node_status.insert(node_id.clone(), *status);
                        }
                    }

                    in_sender.send(event).await.unwrap();
                }
            });

            (in_receiver, spawn)
        };

        let (out_sink, out_spawn) = {
            let (out_sink, mut out_receiver) = mpsc::unbounded();

            let spawn = owned_spawn(async move {
                while let Some(event) = out_receiver.next().await {
                    handle_sink.send(event).await.unwrap();
                }
            });

            (out_sink, spawn)
        };

        TestableTransportHandle {
            cell,
            out_sink,
            in_stream: in_stream.peekable(),
            received_events,
            node_status,
            _in_spawn: in_spawn,
            _out_spawn: out_spawn,
        }
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    pub async fn send_rdv(&mut self, dest: Node, rdv: u64) {
        let frame_builder = Self::empty_message_frame();
        let msg = OutMessage::from_framed_message(&self.cell, ServiceType::Chain, frame_builder)
            .unwrap()
            .with_rdv(rdv.into())
            .with_destination(dest);

        self.send_message(msg).await;
    }

    pub async fn send_stream_msg(
        &mut self,
        dest: Node,
        rdv: u64,
        stream: Box<dyn AsyncRead + Send + Unpin>,
    ) {
        let frame_builder = Self::empty_message_frame();
        let msg = OutMessage::from_framed_message(&self.cell, ServiceType::Chain, frame_builder)
            .unwrap()
            .with_rdv(rdv.into())
            .with_destination(dest)
            .with_stream(stream);

        self.send_message(msg).await;
    }

    pub async fn send_message(&mut self, message: OutMessage) {
        self.out_sink
            .send(OutEvent::Message(message))
            .await
            .unwrap();
    }

    pub async fn recv_msg(&mut self) -> InMessage {
        loop {
            let event = self.in_stream.next().await.unwrap();
            match event {
                InEvent::Message(message) => return message,
                InEvent::NodeStatus(_, _) => {}
            }
        }
    }

    pub async fn recv_rdv(&mut self, rdv: u64) -> InMessage {
        loop {
            let msg = self.recv_msg().await;
            if msg.rendez_vous_id == Some(rdv.into()) {
                return msg;
            }
        }
    }

    pub async fn recv_status(&mut self) -> (NodeId, ConnectionStatus) {
        loop {
            let event = self.in_stream.next().await.unwrap();
            match event {
                InEvent::NodeStatus(node_id, status) => return (node_id, status),
                InEvent::Message(_) => {}
            }
        }
    }

    pub async fn received_count(&self) -> usize {
        self.received_events.load(Ordering::Relaxed)
    }

    pub async fn node_status(&self, node_id: &NodeId) -> Option<ConnectionStatus> {
        let node_status = self.node_status.lock().await;
        node_status.get(node_id).cloned()
    }

    pub async fn has_msg(&mut self) -> Result<bool, Error> {
        let result = futures::future::poll_fn(|cx| -> Poll<bool> {
            let pin_stream = Pin::new(&mut self.in_stream);
            let res = pin_stream.poll_peek(cx).map(|res| res.is_some());

            // poll_peek blocks for next. if it's not ready, there is no message
            match res {
                Poll::Pending => Poll::Ready(false),
                p => p,
            }
        })
        .await;

        Ok(result)
    }

    pub fn empty_message_frame() -> CapnpFrameBuilder<block_operation_header::Owned> {
        let mut frame_builder = CapnpFrameBuilder::<block_operation_header::Owned>::new();
        let _ = frame_builder.get_builder();
        frame_builder
    }
}

#[cfg(test)]
mod test {
    use exocore_core::cell::{FullCell, LocalNode};

    use super::*;

    #[tokio::test]
    async fn send_and_receive() {
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let cell0 = FullCell::generate(node0.clone()).unwrap();
        let node1 = LocalNode::generate();
        let cell1 = FullCell::generate(node1.clone()).unwrap();

        let t0 = hub.get_transport(node0.clone(), ServiceType::Chain);
        let mut t0 = TestableTransportHandle::new(t0, cell0.cell().clone());

        let t1 = hub.get_transport(node1.clone(), ServiceType::Chain);
        let mut t1 = TestableTransportHandle::new(t1, cell1.cell().clone());

        t0.send_rdv(node1.node().clone(), 100).await;

        let msg = t1.recv_msg().await;
        assert_eq!(msg.source.id(), node0.id());
        assert_eq!(msg.rendez_vous_id, Some(100.into()));

        t1.send_rdv(node0.node().clone(), 101).await;

        let msg = t0.recv_msg().await;
        assert_eq!(msg.source.id(), node1.id());
        assert_eq!(msg.rendez_vous_id, Some(101.into()));
    }

    #[tokio::test]
    async fn connection_status_notification() {
        let hub = MockTransport::default();

        let node0 = LocalNode::generate();
        let cell0 = FullCell::generate(node0.clone()).unwrap();

        let t0 = hub.get_transport(node0.clone(), ServiceType::Chain);
        let mut t0 = TestableTransportHandle::new(t0, cell0.cell().clone());

        hub.notify_node_connection_status(node0.id(), ConnectionStatus::Connected);
        let (msg_node, status) = t0.recv_status().await;
        assert_eq!(&msg_node, node0.id());
        assert_eq!(status, ConnectionStatus::Connected);
    }
}
