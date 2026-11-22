use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::AppError;

/// On-disk vault format version for GitVault v2.
pub const VAULT_FORMAT_VERSION: u32 = 2;

const NONCE_LEN: usize = 12;

/// Generates `N` cryptographically secure random bytes.
#[must_use]
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0_u8; N];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

/// Base64 serde adapter for byte vectors.
pub mod b64 {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(deserializer)?;
        STANDARD.decode(text).map_err(serde::de::Error::custom)
    }
}

/// AEAD-encrypted JSON container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedBlob {
    pub v: u32,
    #[serde(with = "b64")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64")]
    pub ciphertext: Vec<u8>,
}

/// Builds associated data binding ciphertext to its project and environment.
#[must_use]
pub fn env_aad(project: &str, environment: &str) -> Vec<u8> {
    format!("gitvault:v2:{project}:{environment}").into_bytes()
}

/// Encrypts plaintext with AES-256-GCM and a fresh 96-bit nonce.
pub fn seal(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<SealedBlob, AppError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce_bytes = random_bytes::<NONCE_LEN>();
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| AppError::Crypto("AES-GCM encryption failed".into()))?;
    Ok(SealedBlob {
        v: VAULT_FORMAT_VERSION,
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypts and authenticates a [`SealedBlob`]. Returns [`AppError::Crypto`] on tamper/key mismatch.
pub fn open(
    key: &[u8; 32],
    sealed: &SealedBlob,
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, AppError> {
    if sealed.v != VAULT_FORMAT_VERSION {
        return Err(AppError::Crypto(format!(
            "unsupported sealed format version {}",
            sealed.v
        )));
    }
    if sealed.nonce.len() != NONCE_LEN {
        return Err(AppError::Crypto("invalid nonce length".into()));
    }
    let plaintext = Aes256Gcm::new(key.into())
        .decrypt(
            Nonce::from_slice(&sealed.nonce),
            Payload {
                msg: &sealed.ciphertext,
                aad,
            },
        )
        .map_err(|_| AppError::Crypto("decryption or integrity verification failed".into()))?;
    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::VaultKey;

    #[test]
    fn seal_and_open_round_trip() {
        let key = VaultKey::generate();
        let aad = env_aad("my-api", "production");
        let sealed = seal(key.as_bytes(), b"SECRET=123", &aad).unwrap();
        assert_eq!(sealed.v, VAULT_FORMAT_VERSION);
        let plain = open(key.as_bytes(), &sealed, &aad).unwrap();
        assert_eq!(plain.as_slice(), b"SECRET=123");
    }

    #[test]
    fn aad_mismatch_fails_decryption() {
        let key = VaultKey::generate();
        let sealed = seal(
            key.as_bytes(),
            b"SECRET=123",
            &env_aad("my-api", "development"),
        )
        .unwrap();
        let err = open(key.as_bytes(), &sealed, &env_aad("my-api", "production")).unwrap_err();
        assert!(matches!(err, AppError::Crypto(_)));
    }

    #[test]
    fn tampered_ciphertext_fails_decryption() {
        let key = VaultKey::generate();
        let aad = env_aad("app", "dev");
        let mut sealed = seal(key.as_bytes(), b"hello", &aad).unwrap();
        sealed.ciphertext[0] ^= 0xFF;
        assert!(open(key.as_bytes(), &sealed, &aad).is_err());
    }
}
