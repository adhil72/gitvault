use std::fmt;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{
    crypto::{open, random_bytes, seal, SealedBlob},
    error::AppError,
};

const WRAP_AAD: &[u8] = b"gitvault:v2:wrap";

/// Symmetric AES-256-GCM vault master key. Zeroized automatically on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey([u8; 32]);

/// Key-encryption key derived from password or recovery key. Zeroized on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Kek([u8; 32]);

impl VaultKey {
    #[must_use]
    pub fn generate() -> Self {
        Self(random_bytes::<32>())
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn to_base64(&self) -> Zeroizing<String> {
        Zeroizing::new(STANDARD.encode(self.0))
    }

    pub fn from_base64(text: &str) -> Result<Self, AppError> {
        let bytes = Zeroizing::new(
            STANDARD
                .decode(text.trim())
                .map_err(|e| AppError::Crypto(format!("invalid base64 key: {e}")))?,
        );
        let arr: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| AppError::Crypto("vault key must be 32 bytes".into()))?;
        Ok(Self(arr))
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

/// Wraps a [`VaultKey`] using a derived KEK.
pub fn wrap_key(kek: &Kek, key: &VaultKey) -> Result<SealedBlob, AppError> {
    seal(kek.as_bytes(), key.as_bytes(), WRAP_AAD)
}

/// Unwraps a [`VaultKey`] using a derived KEK. Maps AEAD failure to [`AppError::WrongPassword`].
pub fn unwrap_key(kek: &Kek, sealed: &SealedBlob) -> Result<VaultKey, AppError> {
    let bytes = open(kek.as_bytes(), sealed, WRAP_AAD).map_err(|_| AppError::WrongPassword)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Crypto("unwrapped key has invalid length".into()))?;
    Ok(VaultKey(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{derive_kek, KdfParams};

    #[test]
    fn wrap_and_unwrap_rejects_wrong_password() {
        let key = VaultKey::generate();
        let salt = random_bytes::<16>();
        let kek = derive_kek("hunter2-long", &salt, &KdfParams::for_tests()).unwrap();
        let wrapped = wrap_key(&kek, &key).unwrap();
        assert_eq!(
            unwrap_key(&kek, &wrapped).unwrap().as_bytes(),
            key.as_bytes()
        );

        let wrong_kek = derive_kek("wrong-password", &salt, &KdfParams::for_tests()).unwrap();
        assert!(matches!(
            unwrap_key(&wrong_kek, &wrapped),
            Err(AppError::WrongPassword)
        ));
    }

    #[test]
    fn debug_redacts_keys() {
        let vk = VaultKey::generate();
        assert_eq!(format!("{vk:?}"), "VaultKey([REDACTED])");
    }
}
