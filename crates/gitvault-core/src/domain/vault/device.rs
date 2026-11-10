//! Trusted devices: a device keeps its own secret and the vault stores the vault key wrapped by
//! it, so the device can unlock without the master password. Entries live in the vault header
//! (no secrets in plain form) and are revoked by deleting the entry.

use gitvault_crypto::{b64, random, unwrap_key, wrap_key, CryptoError, Kek, Sealed, VaultKey};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

const PREFIX: &str = "gvd1";

/// The header's record of one trusted device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    /// The vault key, encrypted with this device's secret.
    pub wrapped: Sealed,
}

/// What a device keeps: its id and the secret that unwraps its entry.
/// Text form: `gvd1.<id>.<base64 secret>`; this is what `GITVAULT_DEVICE_KEY` holds on servers.
pub struct DeviceKey {
    id: String,
    kek: Kek,
}

impl std::fmt::Debug for DeviceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeviceKey({}, [REDACTED])", self.id)
    }
}

impl DeviceKey {
    #[must_use]
    pub fn generate() -> Self {
        Self {
            id: random::bytes::<6>()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            kek: Kek::from_bytes(random::bytes::<32>()),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn to_text(&self) -> String {
        format!("{PREFIX}.{}.{}", self.id, b64::encode(self.kek.as_bytes()))
    }

    /// # Errors
    /// Validation error when the text is not a device key.
    pub fn parse(text: &str) -> Result<Self, CoreError> {
        let invalid = || CoreError::Validation("not a valid device key".into());
        let mut parts = text.trim().splitn(3, '.');
        let (Some(PREFIX), Some(id), Some(secret)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(invalid());
        };
        let bytes: [u8; 32] = b64::decode(secret)
            .map_err(|_| invalid())?
            .try_into()
            .map_err(|_| invalid())?;
        Ok(Self {
            id: id.to_owned(),
            kek: Kek::from_bytes(bytes),
        })
    }

    pub(super) fn wrap(&self, key: &VaultKey) -> Result<Sealed, CryptoError> {
        wrap_key(&self.kek, key)
    }

    pub(super) fn unwrap(&self, sealed: &Sealed) -> Result<VaultKey, CryptoError> {
        unwrap_key(&self.kek, sealed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_form_round_trips() {
        let key = DeviceKey::generate();
        let parsed = DeviceKey::parse(&key.to_text()).unwrap();
        assert_eq!(parsed.id(), key.id());
        assert_eq!(parsed.to_text(), key.to_text());
    }

    #[test]
    fn rejects_garbage() {
        assert!(DeviceKey::parse("nope").is_err());
        assert!(DeviceKey::parse("gvd1.abc.not-base64!").is_err());
        assert!(DeviceKey::parse("gvd1.abc.AAAA").is_err());
    }
}
