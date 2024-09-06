use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex, RwLock, Weak},
    task::{Context, Poll},
    time::Duration,
};

use async_trait::async_trait;
use exocore_core::{
    cell::{Cell, CellNodeRole, Node, NodeId},
    framing::CapnpFrameBuilder,
    futures::interval,
    time::{Clock, ConsistentTimestamp, Instant},
    utils::handle_set::{Handle, HandleSet},
};
use exocore_protos::generated::{
    exocore_store::{EntityQuery, EntityResults, MutationRequest, MutationResult},
    store_transport_capnp::{
        mutation_response, query_response, unwatch_query_request, watched_query_response,
    },
    MessageType,
};
use exocore_transport::{
    transport::ConnectionStatus, InEvent, InMessage, OutEvent, OutMessage, ServiceType,
    TransportServiceHandle,
};
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};

use super::seri::{
    mutation_result_from_response_frame, mutation_to_request_frame,
    query_results_from_response_frame, query_to_request_frame, watched_query_to_request_frame,
};
use crate::{error::Error, mutation::MutationRequestLike, query::WatchToken};

/// This implementation of the AsyncStore allow sending all queries and
/// mutations to a remote node's local store running the `Server` component.
pub struct Client<T>
where
    T: TransportServiceHandle,
{
    config: ClientConfiguration,
    inner: Arc<RwLock<Inner>>,
    transport_handle: T,
    handles: HandleSet,
}

impl<T> Client<T>
where
    T: TransportServiceHandle,
{
    pub fn new(
        config: ClientConfiguration,
        cell: Cell,
        clock: Clock,
        transport_handle: T,
    ) -> Result<Client<T>, Error> {
        let inner = Arc::new(RwLock::new(Inner {
            config,
            cell,
            clock,
            transport_out: None,
            store_node: None,
            store_node_message_queue: Mutex::new(Vec::new()),
            nodes_status: HashMap::new(),
            pending_queries: HashMap::new(),
            watched_queries: HashMap::new(),
            pending_mutations: HashMap::new(),
        }));

        Ok(Client {
            config,
            inner,
            transport_handle,
            handles: HandleSet::new(),
        })
    }

    pub fn get_handle(&self) -> ClientHandle {
        ClientHandle {
            inner: Arc::downgrade(&self.inner),
            handle: self.handles.get_handle(),
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        // create a channel through which we will receive message from our handles to be
        // sent to transport
        let out_receiver = {
            let mut inner = self.inner.write()?;
            let (out_sender, out_receiver) = mpsc::unbounded();
            inner.transport_out = Some(out_sender);
            out_receiver
        };

        // send outgoing messages to transport
        let mut transport_sink = self.transport_handle.get_sink();
        let transport_sender = async move {
            let mut receiver = out_receiver;

            while let Some(item) = receiver.next().await {
                transport_sink.send(item).await?;
            }

            Ok::<(), Error>(())
        };

        // handle incoming messages from transport
        let weak_inner = Arc::downgrade(&self.inner);
        let mut transport_stream = self.transport_handle.get_stream();
        let transport_receiver = async move {
            while let Some(event) = transport_stream.next().await {
                let res = match event {
                    InEvent::Message(msg) => Inner::handle_incoming_message(&weak_inner, msg),
                    InEvent::NodeStatus(node, status) => {
                        Inner::handle_node_status_change(&weak_inner, node, status)
                    }
                };

                if let Err(err) = res {
                    if err.is_fatal() {
                        return Err(err);
                    } else {
                        error!("Couldn't process incoming transport message: {}", err);
                    }
                }
            }

            Ok::<(), Error>(())
        };

        // management timer that checks for timed out queries & register watched queries
        let weak_inner = Arc::downgrade(&self.inner);
        let management_interval = self.config.management_interval;
        let management_timer = async move {
            let mut timer = interval(management_interval);

            loop {
                timer.tick().await;
                Inner::management_timer_process(&weak_inner)?;
            }

            // types the async block
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        };

        futures::select! {
            _ = transport_sender.fuse() => {},
            _ = transport_receiver.fuse() => {},
            _ = management_timer.fuse() => {},
            _ = self.transport_handle.fuse() => {},
            _ = self.handles.on_handles_dropped().fuse() => {},
        };

        info!("Store client dropped");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClientConfiguration {
    pub query_timeout: Duration,
    pub mutation_timeout: Duration,
    pub management_interval: Duration,
    pub watched_register_interval: Duration,
    pub watched_channel_size: usize,
    pub watched_re_register_remote_dropped: bool,
}

/// Keep in sync with application SDK store.
impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {
            query_timeout: Duration::from_secs(10),
            mutation_timeout: Duration::from_secs(5),
            watched_register_interval: Duration::from_secs(10),
            management_interval: Duration::from_secs(1),
            watched_channel_size: 1000,
            watched_re_register_remote_dropped: true,
        }
    }
}

pub(super) struct Inner {
    config: ClientConfiguration,
    cell: Cell,
    clock: Clock,
    transport_out: Option<mpsc::UnboundedSender<OutEvent>>,
    store_node: Option<Node>,
    store_node_message_queue: Mutex<Vec<OutMessage>>,
    nodes_status: HashMap<NodeId, ConnectionStatus>,
    pending_queries: HashMap<ConsistentTimestamp, PendingRequest<EntityResults>>,
    watched_queries: HashMap<ConsistentTimestamp, WatchedQueryRequest>,
    pending_mutations: HashMap<ConsistentTimestamp, PendingRequest<MutationResult>>,
}

impl Inner {
    fn handle_node_status_change(
        weak_inner: &Weak<RwLock<Inner>>,
        node_id: NodeId,
        node_new_status: ConnectionStatus,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        inner.nodes_status.insert(node_id, node_new_status);

        let was_already_connected = inner.store_node.is_some();

        let node_is_connected = |node_id: &NodeId| -> bool {
            let store_node_status = inner.nodes_status.get(node_id);
            store_node_status == Some(&ConnectionStatus::Connected)
        };

        // if the node we are already using for store is connected, we don't have to do
        // anything
        if let Some(store_node) = &inner.store_node {
            if node_is_connected(store_node.id()) {
                // if our current node has just reconnected, we need to make sure watched
                // queries are still registered
                if node_new_status == ConnectionStatus::Connected {
                    inner.send_watched_queries_keepalive(true);
                }

                return Ok(());
            }
        }

        // otherwise we try to find a new store node that is connected
        let new_store_node = {
            let cell_nodes = inner.cell.nodes();
            let cell_nodes_iter = cell_nodes.iter();

            let store_node = cell_nodes_iter
                .with_role(CellNodeRole::Store)
                .find(|n| node_is_connected(n.node().id()));

            store_node.map(|n| n.node()).cloned()
        };
        if let Some(new_store_node) = new_store_node {
            info!("Switching store server to {}", new_store_node);
            inner.store_node = Some(new_store_node);
        }

        if !was_already_connected {
            inner.drain_store_node_message_queue()?;
        }

        inner.send_watched_queries_keepalive(true);

        Ok(())
    }

    fn handle_incoming_message(
        weak_inner: &Weak<RwLock<Inner>>,
        in_message: InMessage,
    ) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        if let Some(store_node) = &inner.store_node {
            if in_message.source.id() != store_node.id() {
                warn!("Got message from a node other than store node (from {} != current {}). Dropping it.", in_message.source, store_node);
                return Ok(());
            }
        }

        let Some(rendez_vous_id) = in_message.rendez_vous_id else {
            return Err(anyhow!(
                "Got an InMessage without a rendez_vous_id (type={:?} from={})",
                in_message.typ,
                in_message.source
            )
            .into());
        };

        match IncomingMessage::parse_incoming_message(&in_message) {
            Ok(IncomingMessage::MutationResponse(mutation)) => {
                if let Some(pending_request) = inner.pending_mutations.remove(&rendez_vous_id) {
                    let _ = pending_request.result_sender.send(Ok(mutation));
                } else {
                    return Err(anyhow!(
                        "Couldn't find pending mutation for mutation response (request_id={:?} type={:?} from={})",
                        rendez_vous_id, in_message.typ, in_message.source
                    ).into());
                }
            }
            Ok(IncomingMessage::QueryResponse(result)) => {
                if let Some(pending_request) = inner.pending_queries.remove(&rendez_vous_id) {
                    let _ = pending_request.result_sender.send(Ok(result));
                } else if let Some(watched_query) = inner.watched_queries.get_mut(&rendez_vous_id) {
                    let _ = watched_query.result_sender.try_send(Ok(result));
                } else {
                    return Err(anyhow!(
                        "Couldn't find pending query for query response (request_id={:?} type={:?} from={})",
                        rendez_vous_id, in_message.typ, in_message.source
                    ).into());
                }
            }
            Err(Error::Remote(err))
                if inner.config.watched_re_register_remote_dropped
                    && err.contains("unregistered") =>
            {
                if let Some(watched_query) = inner.watched_queries.get_mut(&rendez_vous_id) {
                    debug!("Query got unregistered by remote. Re-registering...");
                    watched_query.force_register();
                }
            }
            Err(err) => {
                if let Some(pending_request) = inner.pending_mutations.remove(&rendez_vous_id) {
                    let _ = pending_request.result_sender.send(Err(err));
                } else if let Some(mut watched_query) =
                    inner.watched_queries.remove(&rendez_vous_id)
                {
                    let _ = watched_query.result_sender.try_send(Err(err));
                } else if let Some(pending_request) = inner.pending_queries.remove(&rendez_vous_id)
                {
                    let _ = pending_request.result_sender.send(Err(err));
                }
            }
        }

        Ok(())
    }

    fn management_timer_process(weak_inner: &Weak<RwLock<Inner>>) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        let query_timeout = inner.config.query_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_queries, query_timeout);

        let mutation_timeout = inner.config.mutation_timeout;
        Inner::check_map_requests_timeouts(&mut inner.pending_mutations, mutation_timeout);

        inner.send_watched_queries_keepalive(false);

        Ok(())
    }

    fn send_mutation(
        &mut self,
        request: MutationRequest,
    ) -> Result<oneshot::Receiver<Result<MutationResult, Error>>, Error> {
        let (result_sender, receiver) = oneshot::channel();

        let request_id = self.clock.consistent_time(self.cell.local_node());
        let request_frame = mutation_to_request_frame(request)?;
        let message =
            OutMessage::from_framed_message(&self.cell, ServiceType::Store, request_frame)?
                .with_expiration(Some(Instant::now() + self.config.mutation_timeout))
                .with_rdv(request_id);
        self.send_store_node_message(message)?;

        self.pending_mutations.insert(
            request_id,
            PendingRequest {
                request_id,
                result_sender,
                send_time: Instant::now(),
            },
        );

        Ok(receiver)
    }

    fn send_query(
        &mut self,
        query: EntityQuery,
    ) -> Result<oneshot::Receiver<Result<EntityResults, Error>>, Error> {
        let (result_sender, receiver) = oneshot::channel();

        let request_id = self.clock.consistent_time(self.cell.local_node());
        let request_frame = query_to_request_frame(&query)?;
        let message =
            OutMessage::from_framed_message(&self.cell, ServiceType::Store, request_frame)?
                .with_expiration(Some(Instant::now() + self.config.query_timeout))
                .with_rdv(request_id);
        self.send_store_node_message(message)?;

        self.pending_queries.insert(
            request_id,
            PendingRequest {
                request_id,
                result_sender,
                send_time: Instant::now(),
            },
        );

        Ok(receiver)
    }

    fn watch_query(
        &mut self,
        query: EntityQuery,
    ) -> Result<
        (
            ConsistentTimestamp,
            mpsc::Receiver<Result<EntityResults, Error>>,
        ),
        Error,
    > {
        let (result_sender, result_receiver) = mpsc::channel(self.config.watched_channel_size);
        let request_id = self.clock.consistent_time(self.cell.local_node());
        let watched_query = WatchedQueryRequest {
            request_id,
            result_sender,
            query,
            last_register: Some(Instant::now()),
        };

        self.send_watch_query(&watched_query)?;
        self.watched_queries.insert(request_id, watched_query);

        Ok((request_id, result_receiver))
    }

    fn send_watch_query(&self, watched_query: &WatchedQueryRequest) -> Result<(), Error> {
        let request_frame = watched_query_to_request_frame(&watched_query.query)?;
        let message =
            OutMessage::from_framed_message(&self.cell, ServiceType::Store, request_frame)?
                .with_rdv(watched_query.request_id);

        self.send_store_node_message(message)
    }

    fn send_unwatch_query(&self, token: WatchToken) -> Result<(), Error> {
        let mut frame_builder = CapnpFrameBuilder::<unwatch_query_request::Owned>::new();
        let mut message_builder = frame_builder.get_builder();
        message_builder.set_token(token);

        let message =
            OutMessage::from_framed_message(&self.cell, ServiceType::Store, frame_builder)?;

        self.send_store_node_message(message)
    }

    fn check_map_requests_timeouts<T>(
        requests: &mut HashMap<ConsistentTimestamp, PendingRequest<T>>,
        timeout: Duration,
    ) {
        let mut timed_out_requests = Vec::new();
        for request in requests.values() {
            if request.send_time.elapsed() > timeout {
                timed_out_requests.push(request.request_id);
            }
        }

        for request_id in timed_out_requests {
            if let Some(request) = requests.remove(&request_id) {
                let _ = request
                    .result_sender
                    .send(Err(Error::Timeout(request.send_time.elapsed(), timeout)));
            }
        }
    }

    fn send_watched_queries_keepalive(&mut self, force: bool) {
        let register_interval = self.config.watched_register_interval;

        let mut sent_queries = Vec::new();
        for (token, query) in &self.watched_queries {
            if force
                || query
                    .last_register
                    .map_or(true, |i| i.elapsed() > register_interval)
            {
                if let Err(err) = self.send_watch_query(query) {
                    error!("Couldn't send watch query: {}", err);
                }
                sent_queries.push(*token);
            }
        }

        for token in &sent_queries {
            let query = self.watched_queries.get_mut(token).unwrap();
            query.last_register = Some(Instant::now());
        }
    }

    fn send_store_node_message(&self, message: OutMessage) -> Result<(), Error> {
        let store_node = if let Some(store_node) = &self.store_node {
            store_node.clone()
        } else {
            info!("No store node yet, queueing message");
            let mut store_node_message_queue = self.store_node_message_queue.lock()?;
            store_node_message_queue.push(message);
            return Ok(());
        };

        let transport = self.transport_out.as_ref().ok_or_else(|| {
            Error::Fatal(anyhow!("Tried to send message, but transport_out was none"))
        })?;

        transport
            .unbounded_send(OutEvent::Message(message.with_destination(store_node)))
            .map_err(|_err| {
                Error::Fatal(anyhow!(
                    "Tried to send message, but transport_out channel is closed"
                ))
            })?;

        Ok(())
    }

    fn drain_store_node_message_queue(&self) -> Result<(), Error> {
        let store_node_message_queue: Vec<OutMessage> = {
            let mut store_node_message_queue = self.store_node_message_queue.lock()?;
            std::mem::take(store_node_message_queue.as_mut())
        };

        for message in store_node_message_queue {
            self.send_store_node_message(message)?;
        }

        Ok(())
    }
}

/// Parsed incoming message via transport.
enum IncomingMessage {
    MutationResponse(MutationResult),
    QueryResponse(EntityResults),
}

impl IncomingMessage {
    fn parse_incoming_message(in_message: &InMessage) -> Result<IncomingMessage, Error> {
        match in_message.typ {
            <mutation_response::Owned as MessageType>::MESSAGE_TYPE => {
                let mutation_frame = in_message.get_data_as_framed_message()?;
                let mutation_result = mutation_result_from_response_frame(mutation_frame)?;
                Ok(IncomingMessage::MutationResponse(mutation_result))
            }
            <query_response::Owned as MessageType>::MESSAGE_TYPE
            | <watched_query_response::Owned as MessageType>::MESSAGE_TYPE => {
                let query_frame = in_message.get_data_as_framed_message()?;
                let query_result = query_results_from_response_frame(query_frame)?;
                Ok(IncomingMessage::QueryResponse(query_result))
            }
            other => Err(anyhow!("Received message of unknown type: {}", other).into()),
        }
    }
}

/// Query or mutation request for which we're waiting a response.
struct PendingRequest<T> {
    request_id: ConsistentTimestamp,
    result_sender: oneshot::Sender<Result<T, Error>>,
    send_time: Instant,
}

struct WatchedQueryRequest {
    request_id: ConsistentTimestamp,
    query: EntityQuery,
    result_sender: mpsc::Sender<Result<EntityResults, Error>>,
    last_register: Option<Instant>,
}

impl WatchedQueryRequest {
    fn force_register(&mut self) {
        self.last_register = None;
    }
}

/// Async handle to the store.
#[derive(Clone)]
pub struct ClientHandle {
    inner: Weak<RwLock<Inner>>,
    handle: Handle,
}

impl ClientHandle {
    pub async fn on_start(&self) {
        self.handle.on_set_started().await;
    }

    pub fn store_node(&self) -> Option<Node> {
        let inner = self.inner.upgrade()?;
        let inner = inner.read().ok()?;
        inner.store_node.clone()
    }
}

#[async_trait]
impl crate::store::Store for ClientHandle {
    type WatchedQueryStream = WatchedQueryStream;

    async fn mutate<M: Into<MutationRequestLike> + Send>(
        &self,
        request: M,
    ) -> Result<MutationResult, Error> {
        let result = {
            let inner = self.inner.upgrade().ok_or(Error::Dropped)?;
            let mut inner = inner.write()?;

            inner.send_mutation(request.into().0)?
        };

        result.await.map_err(|_err| Error::Cancelled)?
    }

    async fn query(&self, query: EntityQuery) -> Result<EntityResults, Error> {
        let receiver = {
            let inner = self.inner.upgrade().ok_or(Error::Dropped)?;
            let mut inner = inner.write()?;

            match inner.send_query(query) {
                Ok(receiver) => receiver,
                Err(err) => return Err(err),
            }
        };

        receiver.await.map_err(|_err| Error::Cancelled)?
    }

    fn watched_query(&self, mut query: EntityQuery) -> Result<Self::WatchedQueryStream, Error> {
        let inner = self.inner.upgrade().ok_or(Error::Dropped)?;
        let mut inner = inner.write()?;

        let mut watch_token = query.watch_token;
        if watch_token == 0 {
            watch_token = inner.clock.consistent_time(inner.cell.local_node()).into();
            query.watch_token = watch_token;
        }

        let (request_id, receiver) = inner.watch_query(query)?;

        Ok(WatchedQueryStream {
            inner: self.inner.clone(),
            watch_token: Some(watch_token),
            request_id,
            result: receiver,
        })
    }
}

/// Stream of results for a watched query.
pub struct WatchedQueryStream {
    inner: Weak<RwLock<Inner>>,
    watch_token: Option<WatchToken>,
    request_id: ConsistentTimestamp,
    result: mpsc::Receiver<Result<EntityResults, Error>>,
}

impl Stream for WatchedQueryStream {
    type Item = Result<EntityResults, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.result.poll_next_unpin(cx)
    }
}

impl Drop for WatchedQueryStream {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.watched_queries.remove(&self.request_id);

                if let Some(watch_token) = self.watch_token {
                    let _ = inner.send_unwatch_query(watch_token);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use exocore_core::{
        cell::{FullCell, LocalNode},
        futures::spawn_future,
        tests_utils::expect_eventually,
    };
    use exocore_transport::testing::MockTransport;

    use super::*;
    use crate::{query::QueryBuilder, store::Store};

    #[tokio::test(flavor = "multi_thread")]
    async fn connects_to_online_node() -> anyhow::Result<()> {
        let client_node = LocalNode::generate();
        let full_cell = FullCell::generate(client_node.clone())?;
        let clock = Clock::new();
        let transport = MockTransport::default();

        let mut server_nodes = Vec::new();
        let mut server_transports = Vec::new();
        for _i in 0..2 {
            let node = LocalNode::generate();
            let mut cell_nodes = full_cell.cell().nodes_mut();

            cell_nodes.add(node.node().clone());
            let cell_node = cell_nodes.get_mut(node.id()).unwrap();
            cell_node.add_role(CellNodeRole::Store);

            server_nodes.push(node.clone());

            let transport = transport
                .get_transport(node, ServiceType::Store)
                .into_testable_handle(full_cell.cell().clone());

            server_transports.push(transport);
        }

        let transport_handle = transport.get_transport(client_node, ServiceType::Store);
        let config = ClientConfiguration::default();
        let client = Client::new(config, full_cell.cell().clone(), clock, transport_handle)?;
        let client_inner = client.inner.clone();
        let client_handle = client.get_handle();

        spawn_future(async move {
            let _ = client.run().await;
        });

        tokio::spawn(async move {
            let _ = client_handle.query(QueryBuilder::test(true).build()).await;
        });

        // no node store yet and query should have been queued
        expect_eventually(|| -> bool {
            let inner = client_inner.read().unwrap();
            assert!(inner.store_node.as_ref().is_none());

            let msg_queue = inner.store_node_message_queue.lock().unwrap();
            msg_queue.len() == 1
        });

        // query has not been sent since node nodes are online
        assert!(!server_transports[0].has_msg().await.unwrap());

        // notify that the node 0 is now online
        transport.notify_node_connection_status(server_nodes[0].id(), ConnectionStatus::Connected);

        // should eventually switch to node 0
        expect_eventually(|| -> bool {
            let inner = client_inner.read().unwrap();
            if inner.store_node.is_none() {
                return false;
            }

            inner.store_node.as_ref().unwrap().id() == server_nodes[0].id()
        });

        // node 0 should have receive the query that got queued
        assert!(server_transports[0].has_msg().await.unwrap());

        // node 0 becomes offline, node 1 comes online
        transport
            .notify_node_connection_status(server_nodes[0].id(), ConnectionStatus::Disconnected);
        transport.notify_node_connection_status(server_nodes[1].id(), ConnectionStatus::Connected);

        // should switch to node 1
        expect_eventually(|| -> bool {
            let inner = client_inner.read().unwrap();
            inner.store_node.as_ref().unwrap().id() == server_nodes[1].id()
        });

        Ok(())
    }
}
