use crate::payload::{CreatePayloadRequest, CreatePayloadResponse, Payload, PayloadID};
use reqwest::IntoUrl;
pub use reqwest::Url;

/// Discovery service client.
///
/// The discovery service is a simple REST API on which clients can push temporary payload for which the server
/// generates a random code. Another client can then retrieve that payload by using the generated random code.
/// Once a payload is consumed, it is deleted.
pub struct Client {
    base_uri: Url,
}

impl Client {
    /// Creates a new client instance.
    pub fn new<U: IntoUrl>(base_uri: U) -> Result<Client, Error> {
        Ok(Client {
            base_uri: base_uri.into_url()?,
        })
    }

    /// Creates a new payload on the server. If successfully created, the response contains
    /// a unique identifier that can be used by another client to retrieve the payload.
    pub async fn create(&self, payload: &[u8]) -> Result<CreatePayloadResponse, Error> {
        let b64_payload = base64::encode(payload);
        let create_request = CreatePayloadRequest { data: b64_payload };

        let http_resp = reqwest::Client::builder()
            .build()?
            .post(self.base_uri.clone())
            .json(&create_request)
            .send()
            .await?;

        let create_resp = http_resp.json::<CreatePayloadResponse>().await?;

        Ok(create_resp)
    }

    /// Gets a payload by unique identifier created by the call to `create` by another client.
    pub async fn get(&self, id: PayloadID) -> Result<Vec<u8>, Error> {
        let url = self
            .base_uri
            .join(&format!("/{}", id))
            .expect("Couldn't create URL");
        let http_resp = reqwest::Client::builder().build()?.get(url).send().await?;

        if http_resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }

        let payload = http_resp.json::<Payload>().await?;
        let b64_payload = base64::decode(&payload.data).map_err(|err| {
            error!("Couldn't base64 decode payload data: {}", err);
            Error::InvalidPayload
        })?;

        Ok(b64_payload)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Payload with this id was not found or has expired")]
    NotFound,

    #[error("Received an invalid payload from server")]
    InvalidPayload,
}
