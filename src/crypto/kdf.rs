use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};

use crate::{crypto::Kek, error::AppError};

/// Argon2id parameters stored in the vault header.
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
    /// Fast parameters strictly for automated tests.
    #[cfg(test)]
    #[must_use]
    pub const fn for_tests() -> Self {
        Self {
            memory_kib: 8,
            iterations: 1,
            parallelism: 1,
        }
    }
}

/// Derives a 256-bit KEK from a master password using Argon2id.
pub fn derive_kek(password: &str, salt: &[u8], params: &KdfParams) -> Result<Kek, AppError> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|e| AppError::Crypto(format!("invalid KDF parameters: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut out = [0_u8; 32];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|e| AppError::Crypto(format!("Argon2id KDF failed: {e}")))?;
    Ok(Kek::from_bytes(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_password_and_salt_yield_same_kek() {
        let params = KdfParams::for_tests();
        let a = derive_kek("pw-test", b"0123456789abcdef", &params).unwrap();
        let b = derive_kek("pw-test", b"0123456789abcdef", &params).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }
}
