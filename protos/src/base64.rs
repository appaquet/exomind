use base64::Engine;
use serde::{Deserialize, Deserializer, Serializer};

// Used to allow base64 encoding of bytes in serde
// Ex:
// `#[serde(serialize_with = "crate::protos::base64::as_base64",
//          deserialize_with = "crate::protos::base64::from_base64")
//   ]`
//
// Inspired from https://github.com/serde-rs/serde/issues/661

pub fn as_base64<T, S>(key: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(key.as_ref()))
}

pub fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        base64::engine::general_purpose::STANDARD
            .decode(string)
            .map_err(|err| Error::custom(err.to_string()))
    })
}
