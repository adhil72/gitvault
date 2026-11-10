//! Password-based key derivation (Argon2id).

use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};

use crate::{error::CryptoError, vault_key::Kek};

/// Argon2id cost parameters. Stored in the vault header so they can be raised later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_kib: 64 * 1024,
            iterations: 3,
            parallelism: 1,
        }
    }
}

impl KdfParams {
    /// Very cheap parameters for unit tests only.
    #[must_use]
    pub const fn for_tests() -> Self {
        Self {
            memory_kib: 8,
            iterations: 1,
            parallelism: 1,
        }
    }
}

/// Derives the key-encryption key from a master password and salt.
///
/// # Errors
/// Returns [`CryptoError::Kdf`] for invalid parameters.
pub fn derive_kek(password: &str, salt: &[u8], params: &KdfParams) -> Result<Kek, CryptoError> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut out = [0_u8; 32];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    Ok(Kek::from_bytes(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_give_same_key() {
        let p = KdfParams::for_tests();
        let a = derive_kek("pw", b"0123456789abcdef", &p).unwrap();
        let b = derive_kek("pw", b"0123456789abcdef", &p).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn different_password_or_salt_changes_key() {
        let p = KdfParams::for_tests();
        let base = derive_kek("pw", b"0123456789abcdef", &p).unwrap();
        let other_pw = derive_kek("pw2", b"0123456789abcdef", &p).unwrap();
        let other_salt = derive_kek("pw", b"fedcba9876543210", &p).unwrap();
        assert_ne!(base.as_bytes(), other_pw.as_bytes());
        assert_ne!(base.as_bytes(), other_salt.as_bytes());
    }
}
