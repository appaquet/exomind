use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use rand::Rng;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub struct Pin(u32);

impl From<Pin> for u32 {
    fn from(pin: Pin) -> Self {
        pin.0
    }
}

impl TryFrom<u32> for Pin {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !(100_000_000..=999_999_999).contains(&value) {
            return Err(());
        }

        Ok(Pin(value))
    }
}

impl FromStr for Pin {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pin_int: u32 = s
            .replace(|c: char| c.is_whitespace(), "")
            .parse()
            .map_err(|_| ())?;

        Pin::try_from(pin_int)
    }
}

impl Pin {
    pub fn generate() -> Pin {
        let mut rng = rand::thread_rng();
        rng.gen_range(100_000_000..999_999_999).try_into().unwrap()
    }

    pub fn to_formatted_string(self) -> String {
        let id = self.0;
        let three = id - (id / 1_000) * 1_000;
        let one = id / 1_000_000;
        let two = (id - one * 1_000_000 - three) / 1_000;

        format!("{:03} {:03} {:03}", one, two, three)
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreatePayloadRequest {
    /// Payload
    pub data: String,

    /// If true, a reply token will be generated to be used
    /// to authenticate reply.
    pub expect_reply: bool,
}

/// Random token generated when a payload got accepted used to
/// authenticate the reply.
pub type ReplyToken = u64;

#[derive(Serialize, Deserialize)]
pub struct CreatePayloadResponse {
    /// Pin of the payload
    pub pin: Pin,

    /// Time at which payload will be cleaned up from server
    /// if not consumed before.
    pub expiration: DateTime<Utc>,

    /// If payload expects a reply, pin to be used to reply to
    /// this payload.
    pub reply_pin: Option<Pin>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplyPayloadRequest {
    /// Payload
    pub data: String,

    /// Reply authentication token.
    pub reply_token: ReplyToken,

    /// If true, a reply token will be generated to be used
    /// to authenticate reply.
    pub expect_reply: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Payload {
    /// Pin of the payload
    pub pin: Pin,

    /// Payload
    pub(crate) data: String,

    /// If payload expects a reply, pin to be used to reply to
    /// this payload.
    pub reply_pin: Option<Pin>,

    /// If payload expects a reply, reply token that will be
    /// required in the `ReplyPayloadRequest` to authenticate.
    pub reply_token: Option<ReplyToken>,
}

impl Payload {
    pub fn decode_payload(&self) -> Result<Vec<u8>, PayloadError> {
        let b64_payload = STANDARD.decode(&self.data)?;
        Ok(b64_payload)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PayloadError {
    #[error("Base64 decoding error: {0}")]
    Decode(#[from] base64::DecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_validation() {
        assert!(Pin::try_from(0).is_err());
        assert!(Pin::try_from(99_999_999).is_err());
        assert!(Pin::try_from(100_000_000).is_ok());
        assert!(Pin::try_from(999_999_999).is_ok());
        assert!(Pin::try_from(1_000_000_000).is_err());
    }

    #[test]
    fn pin_formatting() {
        assert_eq!(&Pin(123_456_789).to_formatted_string(), "123 456 789");
        assert_eq!(&Pin(100_000_000).to_formatted_string(), "100 000 000");
        assert_eq!(&Pin(999_999_999).to_formatted_string(), "999 999 999");
    }

    #[test]
    fn pin_string_parsing() {
        assert_eq!("123 456 789".parse().ok(), Some(Pin(123_456_789)));
        assert_eq!("100 000 000".parse().ok(), Some(Pin(100_000_000)));
        assert!("123".parse::<Pin>().is_err());
        assert!("foo".parse::<Pin>().is_err());
        assert_eq!(" 99 9 9 99 999 ".parse().ok(), Some(Pin(999_999_999)));
    }
}
