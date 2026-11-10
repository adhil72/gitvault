use thiserror::Error;

/// Failures of cryptographic operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    /// Authentication failed: wrong key/password, or the data was modified.
    #[error("decryption failed (wrong key or corrupted data)")]
    Decrypt,
    #[error("encryption failed")]
    Encrypt,
    #[error("key derivation failed: {0}")]
    Kdf(String),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}
