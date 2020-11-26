use crate::payload::{CreatePayloadRequest, CreatePayloadResponse, Payload, PayloadID};
use futures::prelude::*;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, StatusCode,
};

mod config;
mod store;
pub use config::ServerConfig;

/// Discovery service server.
///
/// The discovery service is a simple REST API on which clients can push temporary payload for which the server
/// generates a random code. Another client can then retrieve that payload by using the generated random code.
/// Once a payload is consumed, it is deleted.
///
/// Payloads expires after a certain configured delay.
pub struct Server {
    config: ServerConfig,
}

impl Server {
    /// Creates an instance of the server.
    /// Needs to be started using the `start` method in order to start listening for requests.
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Starts the server that will listens for requests and block forever or until failure.
    pub async fn start(&self) -> anyhow::Result<()> {
        let config = self.config;
        let store = store::Store::new(config);

        let server = {
            let store = store.clone();
            let addr = format!("0.0.0.0:{}", config.port).parse()?;
            hyper::Server::bind(&addr).serve(make_service_fn(move |_socket| {
                let store = store.clone();

                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let store = store.clone();

                        async move {
                            let resp = match Self::handle_request(config, req, store).await {
                                Ok(resp) => resp,
                                Err(err) => {
                                    info!("Error handling request: {}", err);
                                    err.to_response()
                                }
                            };

                            Ok::<_, hyper::Error>(resp)
                        }
                    }))
                }
            }))
        };

        let cleaner = {
            let store = store.clone();
            async move {
                let mut interval_stream = tokio::time::interval(config.cleanup_interval);
                while interval_stream.next().await.is_some() {
                    store.cleanup().await;
                }
            }
        };

        info!("Discovery server started on port {}", config.port);
        futures::select! {
            _ = server.fuse() => {},
            _ = cleaner.fuse() => {},
        };

        Ok(())
    }

    async fn handle_request(
        config: ServerConfig,
        req: Request<Body>,
        store: store::Store,
    ) -> Result<Response<Body>, RequestError> {
        let request_type = RequestType::from_method_path(req.method(), req.uri().path())?;
        match request_type {
            RequestType::Post => Self::handle_post(&config, req, store).await,
            RequestType::Get(id) => Self::handle_get(store, id).await,
            RequestType::Options => Self::handle_request_options().await,
        }
    }

    async fn handle_post(
        config: &ServerConfig,
        req: Request<Body>,
        store: store::Store,
    ) -> Result<Response<Body>, RequestError> {
        let req_body_bytes = hyper::body::to_bytes(req.into_body()).await?;

        if req_body_bytes.len() > config.max_payload_size {
            return Err(RequestError::PayloadTooLarge);
        }

        let req_payload = serde_json::from_slice::<CreatePayloadRequest>(req_body_bytes.as_ref())
            .map_err(RequestError::Serialization)?;
        let (id, expiration) = store.push(req_payload.data).await?;

        let resp_payload = CreatePayloadResponse { id, expiration };
        let resp_body_bytes =
            serde_json::to_vec(&resp_payload).map_err(RequestError::Serialization)?;
        let resp_body = Body::from(resp_body_bytes);

        let mut resp = Response::new(resp_body);
        Self::add_cors_headers(&mut resp);
        Ok(resp)
    }

    async fn handle_get(
        store: store::Store,
        id: PayloadID,
    ) -> Result<Response<Body>, RequestError> {
        let data = store.get(id).await.ok_or(RequestError::NotFound)?;

        let resp_payload = Payload { id, data };
        let resp_body_bytes =
            serde_json::to_vec(&resp_payload).map_err(RequestError::Serialization)?;
        let resp_body = Body::from(resp_body_bytes);

        let mut resp = Response::new(resp_body);
        Self::add_cors_headers(&mut resp);
        Ok(resp)
    }

    async fn handle_request_options() -> Result<Response<Body>, RequestError> {
        let mut resp = Response::default();
        Self::add_cors_headers(&mut resp);
        Ok(resp)
    }

    fn add_cors_headers(response: &mut Response<Body>) {
        let headers = response.headers_mut();
        headers.insert(
            hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
            "POST, GET".parse().unwrap(),
        );
        headers.insert(
            hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "*".parse().unwrap(),
        );
    }
}

#[derive(Debug, PartialEq)]
enum RequestType {
    Post,
    Get(PayloadID),
    Options,
}

impl RequestType {
    fn from_method_path(method: &Method, path: &str) -> Result<RequestType, RequestError> {
        match *method {
            Method::POST if path == "/" => Ok(RequestType::Post),
            Method::GET => {
                let id: PayloadID = path.replace("/", "").parse().map_err(|err| {
                    debug!("Couldn't parse path '{}': {}", path, err);
                    RequestError::InvalidRequestType
                })?;
                Ok(RequestType::Get(id))
            }
            Method::OPTIONS => Ok(RequestType::Options),
            _ => {
                debug!(
                    "Received an unknown request: method={} path={}",
                    method, path
                );
                Err(RequestError::InvalidRequestType)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum RequestError {
    #[error("Invalid request type")]
    InvalidRequestType,
    #[error("Payload not found")]
    NotFound,
    #[error("Maximum number of payloads exceeded")]
    Full,
    #[error("Payload is too large")]
    PayloadTooLarge,
    #[error("Invalid request body: {0}")]
    Serialization(#[source] serde_json::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
}

impl RequestError {
    fn to_response(&self) -> Response<Body> {
        let mut resp = Response::default();
        let status = match self {
            RequestError::InvalidRequestType => StatusCode::NOT_FOUND,
            RequestError::NotFound => StatusCode::NOT_FOUND,
            RequestError::Full => StatusCode::INSUFFICIENT_STORAGE,
            RequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            RequestError::Serialization(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        *resp.status_mut() = status;
        resp
    }
}
