use std::{convert::TryInto, time::Duration};

use base64::Engine;
pub use reqwest::Url;
use reqwest::{IntoUrl, StatusCode};
use wasm_timer::Instant;

use crate::payload::{
    CreatePayloadRequest, CreatePayloadResponse, Payload, Pin, ReplyPayloadRequest, ReplyToken,
};

/// Discovery service client.
///
/// The discovery service is a simple REST API on which clients can push
/// temporary payload for which the server generates a random PIN. Another
/// client can then retrieve that payload by using the generated random PIN.
/// Once a payload is consumed, it is deleted.
pub struct Client {
    base_url: Url,
}

impl Client {
    /// Creates a new client instance.
    pub fn new<U: IntoUrl>(base_url: U) -> Result<Client, Error> {
        Ok(Client {
            base_url: base_url.into_url()?,
        })
    }

    /// Creates a new payload on the server. If successfully created, the
    /// response contains a unique pin that can be used by another client to
    /// retrieve the payload.
    pub async fn create(
        &self,
        payload: &[u8],
        expect_reply: bool,
    ) -> Result<CreatePayloadResponse, Error> {
        let b64_payload = base64::engine::general_purpose::STANDARD.encode(payload);
        let create_request = CreatePayloadRequest {
            data: b64_payload,
            expect_reply,
        };

        let http_resp = reqwest::Client::builder()
            .build()?
            .post(self.base_url.clone())
            .json(&create_request)
            .send()
            .await?;

        if http_resp.status() != StatusCode::OK {
            return Err(Error::ServerError(http_resp.status()));
        }

        let create_resp = http_resp.json::<CreatePayloadResponse>().await?;

        Ok(create_resp)
    }

    /// Gets a payload by unique pin created by the call to `create` by another
    /// client.
    pub async fn get<P: TryInto<Pin>>(&self, pin: P) -> Result<Payload, Error> {
        let pin_u32: u32 = pin.try_into().map_err(|_| Error::InvalidPin)?.into();
        let url = self
            .base_url
            .join(&format!("/{}", pin_u32))
            .expect("Couldn't create URL");
        let http_resp = reqwest::Client::builder().build()?.get(url).send().await?;

        match http_resp.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::NOT_FOUND => {
                return Err(Error::NotFound);
            }
            other => {
                return Err(Error::ServerError(other));
            }
        }

        let payload = http_resp.json::<Payload>().await?;

        Ok(payload)
    }

    /// Gets a payload by unique pin created by the call to `create` by another
    /// client and retries fetching if it hasn't been found it until
    /// `timeout`.
    pub async fn get_loop<P: TryInto<Pin>>(
        &self,
        pin: P,
        timeout: Duration,
    ) -> Result<Payload, Error> {
        let pin = pin.try_into().map_err(|_| Error::InvalidPin)?;
        let begin = Instant::now();
        loop {
            match self.get(pin).await {
                Ok(payload) => return Ok(payload),
                Err(Error::NotFound) if begin.elapsed() > timeout => {
                    return Err(Error::NotFound);
                }
                Err(Error::NotFound) if begin.elapsed() < timeout => {
                    debug!("Payload not found on server... Waiting");
                    let _ = wasm_timer::Delay::new(Duration::from_millis(1000)).await;
                }
                Err(other) => return Err(other),
            }
        }
    }

    /// Replies to a payload on the server using the given reply pin and
    /// authentication token. If successfully created, the response contains
    /// can be retrieved using the reply pin.
    pub async fn reply(
        &self,
        reply_pin: Pin,
        reply_token: ReplyToken,
        payload: &[u8],
        expect_reply: bool,
    ) -> Result<CreatePayloadResponse, Error> {
        let pin_u32: u32 = reply_pin.into();
        let url = self
            .base_url
            .join(&format!("/{}", pin_u32))
            .expect("Couldn't create URL");

        let b64_payload = base64::engine::general_purpose::STANDARD.encode(payload);
        let reply_request = ReplyPayloadRequest {
            data: b64_payload,
            expect_reply,
            reply_token,
        };

        let http_resp = reqwest::Client::builder()
            .build()?
            .put(url)
            .json(&reply_request)
            .send()
            .await?;

        match http_resp.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::NOT_FOUND => {
                return Err(Error::NotFound);
            }
            other => {
                return Err(Error::ServerError(other));
            }
        }

        let create_resp = http_resp.json::<CreatePayloadResponse>().await?;

        Ok(create_resp)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Payload with this id was not found or has expired")]
    NotFound,

    #[error("Received an unexpected error from server: code={0}")]
    ServerError(reqwest::StatusCode),

    #[error("Received an invalid payload from server")]
    InvalidPayload,

    #[error("Received an invalid pin")]
    InvalidPin,
}
