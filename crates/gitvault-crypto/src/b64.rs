//! Serde helper that stores byte vectors as standard base64 strings.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Deserializer, Serializer};

/// Encodes bytes as base64.
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

/// Decodes base64 into bytes.
///
/// # Errors
/// Returns the decoding error when the input is not valid base64.
pub fn decode(text: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(text)
}

/// `#[serde(with = "gitvault_crypto::b64")]` serializer.
///
/// # Errors
/// Propagates serializer errors.
#[allow(clippy::ptr_arg)] // serde `with` requires `&Vec<u8>`
pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&encode(bytes))
}

/// `#[serde(with = "gitvault_crypto::b64")]` deserializer.
///
/// # Errors
/// Fails when the string is not valid base64.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    let text = String::deserialize(deserializer)?;
    decode(&text).map_err(serde::de::Error::custom)
}
