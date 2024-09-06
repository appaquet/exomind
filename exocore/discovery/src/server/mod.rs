use std::convert::TryInto;

use futures::prelude::*;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, StatusCode,
};

use crate::payload::{
    CreatePayloadRequest, CreatePayloadResponse, Payload, Pin, ReplyPayloadRequest,
};

mod config;
mod store;
pub use config::ServerConfig;

/// Discovery service server.
///
/// The discovery service is a simple REST API on which clients can push
/// temporary payload for which the server generates a random PIN. Another
/// client can then retrieve that payload by using the generated random PIN.
/// Once a payload is consumed, it is deleted.
///
/// Payloads expires after a certain configured delay.
pub struct Server {
    config: ServerConfig,
}

impl Server {
    /// Creates an instance of the server.
    /// Needs to be started using the `start` method in order to start listening
    /// for requests.
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Starts the server that will listens for requests and block forever or
    /// until failure.
    pub async fn start(&self) -> anyhow::Result<()> {
        let config = self.config;
        let store = store::Store::new(config);

        let server = async {
            let store = store.clone();
            let addr = format!("0.0.0.0:{}", config.port).parse()?;
            let server = hyper::Server::bind(&addr).serve(make_service_fn(move |_socket| {
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
            }));

            Ok::<_, anyhow::Error>(server)
        }
        .await?;

        let cleaner = {
            let store = store.clone();
            async move {
                let mut interval_stream = tokio::time::interval(config.cleanup_interval);
                loop {
                    interval_stream.tick().await;
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
            RequestType::Create => Self::handle_create(&config, req, store).await,
            RequestType::Get(pin) => Self::handle_get(store, pin).await,
            RequestType::Reply(pin) => Self::handle_reply(&config, pin, req, store).await,
            RequestType::Options => Self::handle_request_options().await,
        }
    }

    async fn handle_create(
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

        let (reply_pin, reply_token) = if req_payload.expect_reply {
            let (reply_pin, reply_token) = store.get_reply_token().await;
            (Some(reply_pin), Some(reply_token))
        } else {
            (None, None)
        };

        let (pin, expiration) = store
            .create(req_payload.data, reply_pin, reply_token)
            .await?;

        let resp_payload = CreatePayloadResponse {
            pin,
            expiration,
            reply_pin,
        };
        let resp_body_bytes =
            serde_json::to_vec(&resp_payload).map_err(RequestError::Serialization)?;
        let resp_body = Body::from(resp_body_bytes);

        let mut resp = Response::new(resp_body);
        add_cors_headers(&mut resp);

        Ok(resp)
    }

    async fn handle_get(store: store::Store, pin: Pin) -> Result<Response<Body>, RequestError> {
        let stored_payload = store.get(pin).await.ok_or(RequestError::NotFound)?;

        let resp_payload = Payload {
            pin,
            data: stored_payload.data,
            reply_pin: stored_payload.reply_pin,
            reply_token: stored_payload.reply_token,
        };
        let resp_body_bytes =
            serde_json::to_vec(&resp_payload).map_err(RequestError::Serialization)?;
        let resp_body = Body::from(resp_body_bytes);

        let mut resp = Response::new(resp_body);
        add_cors_headers(&mut resp);

        Ok(resp)
    }

    async fn handle_reply(
        config: &ServerConfig,
        pin: Pin,
        req: Request<Body>,
        store: store::Store,
    ) -> Result<Response<Body>, RequestError> {
        let req_body_bytes = hyper::body::to_bytes(req.into_body()).await?;

        if req_body_bytes.len() > config.max_payload_size {
            return Err(RequestError::PayloadTooLarge);
        }

        let req_payload = serde_json::from_slice::<ReplyPayloadRequest>(req_body_bytes.as_ref())
            .map_err(RequestError::Serialization)?;

        let (reply_pin, reply_token) = if req_payload.expect_reply {
            let (reply_pin, reply_token) = store.get_reply_token().await;
            (Some(reply_pin), Some(reply_token))
        } else {
            (None, None)
        };

        let expiration = store
            .push_reply(
                pin,
                req_payload.reply_token,
                req_payload.data,
                reply_pin,
                reply_token,
            )
            .await?;

        let resp_payload = CreatePayloadResponse {
            pin,
            expiration,
            reply_pin,
        };
        let resp_body_bytes =
            serde_json::to_vec(&resp_payload).map_err(RequestError::Serialization)?;
        let resp_body = Body::from(resp_body_bytes);

        let mut resp = Response::new(resp_body);
        add_cors_headers(&mut resp);

        Ok(resp)
    }

    async fn handle_request_options() -> Result<Response<Body>, RequestError> {
        let mut resp = Response::default();
        add_cors_headers(&mut resp);
        Ok(resp)
    }
}

#[derive(Debug, PartialEq)]
enum RequestType {
    Create,
    Get(Pin),
    Reply(Pin),
    Options,
}

impl RequestType {
    fn from_method_path(method: &Method, path: &str) -> Result<RequestType, RequestError> {
        match *method {
            Method::POST if path == "/" => Ok(RequestType::Create),
            Method::GET => {
                let pin = Self::parse_path_pin(method, path)?;
                Ok(RequestType::Get(pin))
            }
            Method::PUT => {
                let pin = Self::parse_path_pin(method, path)?;
                Ok(RequestType::Reply(pin))
            }
            Method::OPTIONS => Ok(RequestType::Options),
            _ => Err(RequestError::InvalidRequestType(
                method.clone(),
                path.to_string(),
            )),
        }
    }

    fn parse_path_pin(method: &Method, path: &str) -> Result<Pin, RequestError> {
        let pin: Pin = path
            .replace('/', "")
            .parse::<u32>()
            .map_err(|err| {
                debug!("Couldn't parse path '{}': {}", path, err);
                RequestError::InvalidRequestType(method.clone(), path.to_string())
            })?
            .try_into()
            .map_err(|_| {
                debug!("Couldn't parse pin in path '{}'", path);
                RequestError::InvalidRequestType(method.clone(), path.to_string())
            })?;

        Ok(pin)
    }
}

#[derive(Debug, thiserror::Error)]
enum RequestError {
    #[error("Invalid request type: {0} {1}")]
    InvalidRequestType(Method, String),
    #[error("Payload not found")]
    NotFound,
    #[error("Invalid reply pin or unique token")]
    InvalidReply,
    #[error("Maximum number of payloads exceeded")]
    Full,
    #[error("Payload is too large")]
    PayloadTooLarge,
    #[error("Invalid request body: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
}

impl RequestError {
    fn to_response(&self) -> Response<Body> {
        let mut resp = Response::default();
        let status = match self {
            RequestError::InvalidRequestType(_, _) => StatusCode::NOT_FOUND,
            RequestError::NotFound => StatusCode::NOT_FOUND,
            RequestError::InvalidReply => StatusCode::UNAUTHORIZED,
            RequestError::Full => StatusCode::INSUFFICIENT_STORAGE,
            RequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            RequestError::Serialization(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        add_cors_headers(&mut resp);

        *resp.status_mut() = status;
        resp
    }
}

fn add_cors_headers(response: &mut Response<Body>) {
    let headers = response.headers_mut();
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
        "POST, PUT, GET".parse().unwrap(),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        "*".parse().unwrap(),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Origin, X-Requested-With, Content-Type, Accept, Authorization"
            .parse()
            .unwrap(),
    );
}
