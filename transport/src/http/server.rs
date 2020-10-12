use super::{
    handles::ServiceHandle, handles::ServiceHandles, requests::RequestTracker,
    requests::TrackedRequest, HTTPTransportConfig, HTTPTransportServiceHandle,
};

use crate::Error;
use crate::{transport::ConnectionID, InMessage, OutEvent, OutMessage, ServiceType};

use exocore_core::{
    capnp,
    cell::{Cell, CellNodes, LocalNode, Node},
    framing::{CapnpFrameBuilder, FrameBuilder},
    futures::block_on,
    protos::generated::store_transport_capnp::mutation_request,
    protos::generated::store_transport_capnp::mutation_response,
    protos::generated::store_transport_capnp::query_request,
    protos::generated::store_transport_capnp::query_response,
    sec::auth_token::AuthToken,
    time::Clock,
    utils::handle_set::HandleSet,
};

use futures::{channel::mpsc, lock::Mutex, FutureExt, StreamExt};
use hyper::{
    service::{make_service_fn, service_fn},
    StatusCode,
};
use hyper::{Body, Request, Response, Server};

use std::{borrow::Cow, sync::Arc};

/// Unidirectional HTTP transport server used for request-response type of
/// communication by clients for which a full libp2p transport is impossible.
///
/// Since it doesn't run a full fledge transport, authentication is achieved
/// through a generated `AuthToken` signed by the public key of a node of the
/// cell.
///
/// At the moment, this transport is only used for entity queries and mutations.
pub struct HTTPTransportServer {
    local_node: LocalNode,
    config: HTTPTransportConfig,
    clock: Clock,
    service_handles: Arc<Mutex<ServiceHandles>>,
    handle_set: HandleSet,
}

impl HTTPTransportServer {
    /// Creates a new HTTP server with the given configuration and clock.
    pub fn new(
        local_node: LocalNode,
        config: HTTPTransportConfig,
        clock: Clock,
    ) -> HTTPTransportServer {
        HTTPTransportServer {
            local_node,
            config,
            clock,
            service_handles: Default::default(),
            handle_set: Default::default(),
        }
    }

    /// Get a transport handle that will be used by services. This handle can
    /// only be used to receive messages and reply to them.
    pub fn get_handle(
        &mut self,
        cell: Cell,
        service_type: ServiceType,
    ) -> Result<HTTPTransportServiceHandle, Error> {
        let (in_sender, in_receiver) = mpsc::channel(self.config.handle_in_channel_size);
        let (out_sender, out_receiver) = mpsc::channel(self.config.handle_out_channel_size);

        // Register new handle and its streams
        let mut service_handles = block_on(self.service_handles.lock());
        service_handles.push_handle(cell.clone(), service_type, in_sender, out_receiver);

        info!(
            "Registering transport for cell {} and service type {:?}",
            cell, service_type
        );

        Ok(HTTPTransportServiceHandle {
            cell_id: cell.id().clone(),
            service_type,
            inner: Arc::downgrade(&self.service_handles),
            sink: Some(out_sender),
            stream: Some(in_receiver),
            handle: self.handle_set.get_handle(),
        })
    }

    /// Runs the HTTP server and returns when it's done.
    pub async fn run(self) -> Result<(), Error> {
        let request_tracker = Arc::new(RequestTracker::new(self.config.clone()));

        // Listen on all addresess
        let servers = {
            let mut futures = Vec::new();
            for listen_url in &self.config.listen_addresses(&self.local_node)? {
                let host = listen_url.domain().unwrap_or("0.0.0.0");
                let port = listen_url.port().unwrap_or(80);
                let addr_res = format!("{}:{}", host, port).parse();
                let addr = match addr_res {
                    Ok(addr) => addr,
                    Err(err) => {
                        error!(
                            "Couldn't extract and parse listen address from url {} ({}:{}): {}",
                            listen_url, host, port, err
                        );
                        continue;
                    }
                };

                info!("Starting a server on {} ({})", addr, listen_url);

                let request_tracker = request_tracker.clone();
                let service_handles = self.service_handles.clone();
                let clock = self.clock.clone();

                let server = Server::bind(&addr).serve(make_service_fn(move |_socket| {
                    let request_tracker = request_tracker.clone();
                    let service_handles = service_handles.clone();
                    let clock = clock.clone();
                    async move {
                        Ok::<_, hyper::Error>(service_fn(move |req| {
                            let request_tracker = request_tracker.clone();
                            let service_handles = service_handles.clone();
                            let clock = clock.clone();

                            async {
                                let resp =
                                    handle_request(request_tracker, service_handles, clock, req)
                                        .await;

                                let resp = match resp {
                                    Ok(resp) => resp,
                                    Err(err) => {
                                        error!("Error handling request: {}", err);
                                        err.to_response()
                                    }
                                };

                                Ok::<_, hyper::Error>(resp)
                            }
                        }))
                    }
                }));

                futures.push(server);
            }

            futures::future::join_all(futures)
        };

        // Takes care of outgoing messages from services to be dispatched to connections
        let handles_dispatcher = {
            let services = self.service_handles.clone();
            let request_tracker = request_tracker.clone();

            async move {
                let mut inner = services.lock().await;

                let mut futures = Vec::new();
                for service_channels in inner.service_handles.values_mut() {
                    let mut out_receiver = service_channels
                        .out_receiver
                        .take()
                        .expect("Out receiver of one service was already consumed");

                    let connections = request_tracker.clone();
                    futures.push(async move {
                        while let Some(event) = out_receiver.next().await {
                            let  OutEvent::Message(message) = event;
                            let connection_id = match message.connection {
                                Some(ConnectionID::HTTPServer(id)) => id,
                                _ => {
                                    warn!("Couldn't find connection id in message to be send back to connection");
                                    continue;
                                }
                            };

                            connections.reply(connection_id, message).await;
                        }
                    });
                }
                futures::future::join_all(futures)
            }
            .await
        };

        info!("HTTP transport now running");
        futures::select! {
            _ = servers.fuse() => (),
            _ = handles_dispatcher.fuse() => (),
            _ = self.handle_set.on_handles_dropped().fuse() => (),
        };
        info!("HTTP transport is done");

        Ok(())
    }
}

/// Handles a single request from a connection by sending it to the appropriate
/// service.
async fn handle_request(
    request_tracker: Arc<RequestTracker>,
    service_handles: Arc<Mutex<ServiceHandles>>,
    clock: Clock,
    req: Request<Body>,
) -> Result<Response<Body>, RequestError> {
    let request_type = RequestType::from_url_path(req.uri().path()).map_err(|err| {
        error!("Invalid request type with path {}", req.uri().path());
        err
    })?;

    // Authentify the request using the authentication token and extract cell & node
    // from it
    let auth_token_str = read_authorization_token(&req)?;
    let auth_token = AuthToken::decode_base58_string(&auth_token_str).map_err(|err| {
        warn!(
            "Unauthorized request for {:?} using token {}: {}",
            request_type, auth_token_str, err
        );
        RequestError::Unauthorized
    })?;

    let mut services = service_handles.lock().await;
    let service = services
        .get_handle(auth_token.cell_id(), request_type.service_type())
        .ok_or_else(|| {
            warn!("Cell {} not found for request", auth_token.cell_id());
            RequestError::InvalidRequestType
        })?;

    let from_node = {
        let cell_nodes = service.cell.nodes();
        cell_nodes
            .get(auth_token.node_id())
            .map(|c| c.node().clone())
            .ok_or_else(|| {
                warn!(
                    "Node {} not found in cell {} for request",
                    auth_token.node_id(),
                    auth_token.cell_id()
                );
                RequestError::InvalidRequestType
            })?
    };

    match request_type {
        RequestType::EntitiesQuery => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let tracked_request = request_tracker.push().await;
            let cell = service.cell.clone();

            send_entity_query(
                body_bytes.as_ref(),
                &clock,
                from_node,
                service,
                &tracked_request,
            )
            .await?;

            drop(services); // drop handles to release lock while we wait for answer

            Ok(receive_entity_query(&cell, tracked_request).await?)
        }
        RequestType::EntitiesMutation => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let tracked_request = request_tracker.push().await;
            let cell = service.cell.clone();

            send_entity_mutation(
                body_bytes.as_ref(),
                &clock,
                from_node,
                service,
                &tracked_request,
            )
            .await?;

            drop(services); // drop handles to release lock while we wait for answer

            Ok(receive_entity_mutation(&cell, tracked_request).await?)
        }
    }
}

async fn send_entity_query(
    body_bytes: &[u8],
    clock: &Clock,
    from_node: Node,
    service: &mut ServiceHandle,
    tracked_request: &TrackedRequest,
) -> Result<(), RequestError> {
    let local_node = service.cell.local_node().node().clone();

    let mut frame_builder = CapnpFrameBuilder::<query_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();
    msg_builder.set_request(body_bytes);

    let message =
        OutMessage::from_framed_message(&service.cell, ServiceType::Store, frame_builder)?
            .with_to_node(local_node)
            .with_rendez_vous_id(clock.consistent_time(service.cell.local_node()))
            .with_connection(ConnectionID::HTTPServer(tracked_request.id()))
            .to_in_message(from_node)?;

    service.send_message(message)?;

    Ok(())
}

async fn receive_entity_query(
    cell: &Cell,
    tracked_request: TrackedRequest,
) -> Result<Response<Body>, RequestError> {
    let local_node = cell.local_node().node().clone();

    let response_message = tracked_request
        .get_response_or_timeout()
        .await
        .map_err(|_| RequestError::Server("Couldn't receive response from handle".to_string()))?;

    let message_envelope = response_message.envelope_builder.as_owned_frame();
    let message = InMessage::from_node_and_frame(local_node, message_envelope)?;
    let result_message = message.get_data_as_framed_message::<query_response::Owned>()?;
    let result_reader = result_message.get_reader()?;

    if !result_reader.has_error() {
        let body = Body::from(result_reader.get_response()?.to_vec());
        Ok(Response::new(body))
    } else {
        Err(RequestError::Query)
    }
}

async fn send_entity_mutation(
    body_bytes: &[u8],
    clock: &Clock,
    from_node: Node,
    service: &mut ServiceHandle,
    tracked_request: &TrackedRequest,
) -> Result<(), RequestError> {
    let local_node = service.cell.local_node().node().clone();

    let mut frame_builder = CapnpFrameBuilder::<mutation_request::Owned>::new();
    let mut msg_builder = frame_builder.get_builder();
    msg_builder.set_request(body_bytes);

    let message =
        OutMessage::from_framed_message(&service.cell, ServiceType::Store, frame_builder)?
            .with_to_node(local_node)
            .with_rendez_vous_id(clock.consistent_time(service.cell.local_node()))
            .with_connection(ConnectionID::HTTPServer(tracked_request.id()))
            .to_in_message(from_node)?;

    service.send_message(message)?;

    Ok(())
}

async fn receive_entity_mutation(
    cell: &Cell,
    tracked_request: TrackedRequest,
) -> Result<Response<Body>, RequestError> {
    let local_node = cell.local_node().node().clone();

    let response_message = tracked_request
        .get_response_or_timeout()
        .await
        .map_err(|_| RequestError::Server("Couldn't receive response from handle".to_string()))?;

    let message_envelope = response_message.envelope_builder.as_owned_frame();
    let message = InMessage::from_node_and_frame(local_node, message_envelope)?;
    let result_message = message.get_data_as_framed_message::<mutation_response::Owned>()?;
    let result_reader = result_message.get_reader()?;

    if !result_reader.has_error() {
        let body = Body::from(result_reader.get_response()?.to_vec());
        Ok(Response::new(body))
    } else {
        Err(RequestError::Query)
    }
}

fn read_authorization_token(request: &Request<Body>) -> Result<String, RequestError> {
    let pq = request.uri();
    let path_and_query = pq.path_and_query().ok_or(RequestError::Unauthorized)?;
    let query = path_and_query.query().ok_or(RequestError::Unauthorized)?;

    let params = url::form_urlencoded::parse(query.as_bytes());
    let token = get_query_token(params).ok_or(RequestError::Unauthorized)?;

    Ok(token.to_string())
}

fn get_query_token(pairs: url::form_urlencoded::Parse) -> Option<Cow<str>> {
    for (key, value) in pairs {
        if key == "token" {
            return Some(value);
        }
    }

    None
}

/// Type of an incoming HTTP request.
#[derive(Debug, PartialEq)]
enum RequestType {
    EntitiesQuery,
    EntitiesMutation,
}

impl RequestType {
    fn from_url_path(path: &str) -> Result<RequestType, RequestError> {
        if path == "/entities/query" {
            Ok(RequestType::EntitiesQuery)
        } else if path == "/entities/mutate" {
            Ok(RequestType::EntitiesMutation)
        } else {
            Err(RequestError::InvalidRequestType)
        }
    }

    fn service_type(&self) -> ServiceType {
        match self {
            RequestType::EntitiesQuery => ServiceType::Store,
            RequestType::EntitiesMutation => ServiceType::Store,
        }
    }
}

/// Request related error.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Invalid request type")]
    InvalidRequestType,
    #[error("Request unauthorized")]
    Unauthorized,
    #[error("Query error")]
    Query,
    #[error("Internal server error: {0}")]
    Server(String),
    #[error("Transport error: {0}")]
    Transport(#[from] crate::Error),
    #[error("Capnp serialization error: {0}")]
    Serialization(#[from] capnp::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
}

impl RequestError {
    fn to_response(&self) -> Response<Body> {
        let mut resp = Response::default();
        let status = match self {
            RequestError::InvalidRequestType => StatusCode::NOT_FOUND,
            RequestError::Unauthorized => StatusCode::UNAUTHORIZED,
            RequestError::Query => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        *resp.status_mut() = status;
        resp
    }
}
