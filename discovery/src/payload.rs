use std::convert::TryFrom;

use chrono::{DateTime, Utc};

#[derive(
    Clone, Copy, Ord, PartialOrd, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug, Hash,
)]
pub struct Pin(u32);

impl Into<u32> for Pin {
    fn into(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for Pin {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 100_000_000 || value > 999_999_999 {
            return Err(());
        }

        Ok(Pin(value))
    }
}

impl Pin {
    pub fn to_formatted_string(&self) -> String {
        let id = self.0;
        let three = id - (id / 1_000) * 1_000;
        let one = id / 1_000_000;
        let two = (id - one * 1_000_000 - three) / 1_000;

        format!("{:03} {:03} {:03}", one, two, three)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreatePayloadRequest {
    pub data: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreatePayloadResponse {
    pub id: Pin,
    pub expiration: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Payload {
    pub id: Pin,
    pub data: String,
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
}
