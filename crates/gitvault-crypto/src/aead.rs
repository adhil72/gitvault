//! AES-256-GCM authenticated encryption with associated data.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};

use crate::{b64, error::CryptoError, random};

/// Length of the AES-GCM nonce in bytes.
pub const NONCE_LEN: usize = 12;

/// Ciphertext together with the nonce used to produce it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sealed {
    #[serde(with = "b64")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64")]
    pub ciphertext: Vec<u8>,
}

/// Encrypts `plaintext` with a fresh random nonce, binding `aad` to the result.
///
/// # Errors
/// Returns [`CryptoError::Encrypt`] if encryption fails.
pub fn seal(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Sealed, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce_bytes = random::bytes::<NONCE_LEN>();
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt)?;
    Ok(Sealed {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypts and authenticates `sealed`. `aad` must match the value used in [`seal`].
///
/// # Errors
/// Returns [`CryptoError::Decrypt`] on a wrong key, wrong `aad`, or tampering.
pub fn open(key: &[u8; 32], sealed: &Sealed, aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if sealed.nonce.len() != NONCE_LEN {
        return Err(CryptoError::InvalidFormat("nonce must be 12 bytes".into()));
    }
    Aes256Gcm::new(key.into())
        .decrypt(
            Nonce::from_slice(&sealed.nonce),
            Payload {
                msg: &sealed.ciphertext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7; 32];

    #[test]
    fn round_trips() {
        let sealed = seal(&KEY, b"hello", b"aad").unwrap();
        assert_eq!(open(&KEY, &sealed, b"aad").unwrap(), b"hello");
    }

    #[test]
    fn uses_a_fresh_nonce_each_time() {
        let a = seal(&KEY, b"same", b"").unwrap();
        let b = seal(&KEY, b"same", b"").unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn rejects_wrong_key() {
        let sealed = seal(&KEY, b"hello", b"").unwrap();
        assert_eq!(open(&[8; 32], &sealed, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn rejects_wrong_associated_data() {
        let sealed = seal(&KEY, b"hello", b"one").unwrap();
        assert_eq!(open(&KEY, &sealed, b"two"), Err(CryptoError::Decrypt));
    }

    #[test]
    fn detects_tampering() {
        let mut sealed = seal(&KEY, b"hello", b"").unwrap();
        sealed.ciphertext[0] ^= 1;
        assert_eq!(open(&KEY, &sealed, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn serializes_as_base64_json() {
        let sealed = seal(&KEY, b"x", b"").unwrap();
        let json = serde_json::to_string(&sealed).unwrap();
        assert!(json.contains("\"nonce\":\""));
        let back: Sealed = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sealed);
    }
}
