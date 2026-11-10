//! Vault key (encrypts secrets) and key-encryption key (wraps the vault key).

use std::fmt;

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{aead, b64, error::CryptoError, random, Sealed};

const WRAP_AAD: &[u8] = b"gitvault:wrap:v1";

/// Key that encrypts every secret file of one vault. Zeroized on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey([u8; 32]);

/// Key-encryption key derived from the master password or the recovery key.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Kek([u8; 32]);

impl VaultKey {
    #[must_use]
    pub fn generate() -> Self {
        Self(random::bytes::<32>())
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Base64 form used only for short-lived session tokens.
    #[must_use]
    pub fn to_base64(&self) -> String {
        b64::encode(&self.0)
    }

    /// Parses the output of [`VaultKey::to_base64`].
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidFormat`] for malformed input.
    pub fn from_base64(text: &str) -> Result<Self, CryptoError> {
        let bytes = b64::decode(text).map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;
        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidFormat("key must be 32 bytes".into()))?;
        Ok(Self(array))
    }
}

impl Kek {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for VaultKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VaultKey([REDACTED])")
    }
}

impl fmt::Debug for Kek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Kek([REDACTED])")
    }
}

/// Encrypts the vault key under a KEK.
///
/// # Errors
/// Returns [`CryptoError::Encrypt`] on failure.
pub fn wrap_key(kek: &Kek, key: &VaultKey) -> Result<Sealed, CryptoError> {
    aead::seal(kek.as_bytes(), key.as_bytes(), WRAP_AAD)
}

/// Decrypts a wrapped vault key. Fails with [`CryptoError::Decrypt`] for a wrong KEK.
///
/// # Errors
/// Returns [`CryptoError::Decrypt`] when the KEK is wrong or the data is corrupted.
pub fn unwrap_key(kek: &Kek, sealed: &Sealed) -> Result<VaultKey, CryptoError> {
    let bytes = aead::open(kek.as_bytes(), sealed, WRAP_AAD)?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| CryptoError::InvalidFormat("wrapped key size".into()))?;
    Ok(VaultKey(array))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_then_unwrap_returns_same_key() {
        let kek = Kek::from_bytes([1; 32]);
        let key = VaultKey::generate();
        let wrapped = wrap_key(&kek, &key).unwrap();
        assert_eq!(
            unwrap_key(&kek, &wrapped).unwrap().as_bytes(),
            key.as_bytes()
        );
    }

    #[test]
    fn wrong_kek_fails() {
        let key = VaultKey::generate();
        let wrapped = wrap_key(&Kek::from_bytes([1; 32]), &key).unwrap();
        assert!(matches!(
            unwrap_key(&Kek::from_bytes([2; 32]), &wrapped),
            Err(CryptoError::Decrypt)
        ));
    }

    #[test]
    fn debug_never_prints_key_material() {
        assert!(!format!("{:?}", VaultKey::generate()).contains(char::is_numeric));
    }

    #[test]
    fn base64_round_trip() {
        let key = VaultKey::generate();
        let back = VaultKey::from_base64(&key.to_base64()).unwrap();
        assert_eq!(back.as_bytes(), key.as_bytes());
    }
}
