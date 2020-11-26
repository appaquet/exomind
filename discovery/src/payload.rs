use chrono::{DateTime, Utc};

pub type PayloadID = u32;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreatePayloadRequest {
    pub data: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreatePayloadResponse {
    pub id: PayloadID,
    pub expiration: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Payload {
    pub id: PayloadID,
    pub data: String,
}
