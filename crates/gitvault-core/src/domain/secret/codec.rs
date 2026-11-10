//! Encrypting and decrypting secret files.
//!
//! A file is a small JSON envelope `{ "v": 1, "nonce": "...", "ciphertext": "..." }`.
//! The vault id and secret id are bound to the ciphertext as associated data, so
//! a file copied between vaults or renamed fails to decrypt.

use gitvault_crypto::{open, seal, Sealed, VaultKey};
use serde::{Deserialize, Serialize};

use super::SecretPayload;
use crate::{
    domain::ids::{SecretId, VaultId},
    error::CoreError,
};

const FILE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Envelope {
    v: u32,
    #[serde(flatten)]
    sealed: Sealed,
}

fn associated_data(vault: &VaultId, secret: &SecretId) -> Vec<u8> {
    format!("gitvault:v{FILE_VERSION}:{vault}:{secret}").into_bytes()
}

/// Encrypts a payload into the bytes of a secret file.
///
/// # Errors
/// Fails if serialization or encryption fails.
pub fn encrypt(
    key: &VaultKey,
    vault: &VaultId,
    secret: &SecretId,
    payload: &SecretPayload,
) -> Result<Vec<u8>, CoreError> {
    let plaintext = serde_json::to_vec(payload).map_err(|e| CoreError::Internal(e.to_string()))?;
    let sealed = seal(key.as_bytes(), &plaintext, &associated_data(vault, secret))?;
    serde_json::to_vec_pretty(&Envelope {
        v: FILE_VERSION,
        sealed,
    })
    .map_err(|e| CoreError::Internal(e.to_string()))
}

/// Decrypts the bytes of a secret file.
///
/// # Errors
/// Fails for a wrong key, tampered or foreign files, and malformed JSON.
pub fn decrypt(
    key: &VaultKey,
    vault: &VaultId,
    secret: &SecretId,
    bytes: &[u8],
) -> Result<SecretPayload, CoreError> {
    let envelope: Envelope = serde_json::from_slice(bytes)
        .map_err(|e| CoreError::Storage(format!("malformed secret file: {e}")))?;
    if envelope.v != FILE_VERSION {
        return Err(CoreError::Storage(format!(
            "unsupported secret file version {}",
            envelope.v
        )));
    }
    let plaintext = open(
        key.as_bytes(),
        &envelope.sealed,
        &associated_data(vault, secret),
    )
    .map_err(|_| {
        CoreError::Storage("secret file failed authentication (tampered or wrong vault)".into())
    })?;
    serde_json::from_slice(&plaintext)
        .map_err(|e| CoreError::Storage(format!("malformed secret: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::secret::{SecretKind, SecretValue};

    fn payload() -> SecretPayload {
        SecretPayload {
            name: "API_KEY".into(),
            kind: SecretKind::ApiKey,
            environment: Some("Production".into()),
            tags: vec!["billing".into()],
            value: SecretValue::new("sk_live_123"),
            notes: String::new(),
            expires_at: None,
            remind_before_days: None,
            favorite: false,
            created_at: 1,
            updated_at: 1,
            version: 1,
        }
    }

    fn ids() -> (VaultId, SecretId) {
        (
            VaultId::parse("v1").unwrap(),
            SecretId::parse("s1").unwrap(),
        )
    }

    #[test]
    fn round_trips_and_hides_plaintext() {
        let key = VaultKey::generate();
        let (v, s) = ids();
        let bytes = encrypt(&key, &v, &s, &payload()).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("sk_live_123"));
        assert!(!String::from_utf8_lossy(&bytes).contains("API_KEY"));
        assert_eq!(decrypt(&key, &v, &s, &bytes).unwrap(), payload());
    }

    #[test]
    fn file_cannot_be_moved_to_another_secret_or_vault() {
        let key = VaultKey::generate();
        let (v, s) = ids();
        let bytes = encrypt(&key, &v, &s, &payload()).unwrap();
        assert!(decrypt(&key, &v, &SecretId::parse("s2").unwrap(), &bytes).is_err());
        assert!(decrypt(&key, &VaultId::parse("v2").unwrap(), &s, &bytes).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let (v, s) = ids();
        let bytes = encrypt(&VaultKey::generate(), &v, &s, &payload()).unwrap();
        assert!(decrypt(&VaultKey::generate(), &v, &s, &bytes).is_err());
    }
}
