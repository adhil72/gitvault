//! Pure cryptographic primitives for GitVault.
//!
//! No I/O and no dependency on other GitVault crates. See
//! `docs/architecture.md` section 6 for the key hierarchy:
//!
//! ```text
//! master password --Argon2id--> KEK --unwrap--> VaultKey --AES-256-GCM--> secret files
//! recovery key    --HKDF------> KEK --unwrap--> VaultKey
//! ```

pub mod aead;
pub mod b64;
pub mod error;
pub mod kdf;
pub mod random;
pub mod recovery;
pub mod vault_key;

pub use aead::{open, seal, Sealed};
pub use error::CryptoError;
pub use kdf::{derive_kek, KdfParams};
pub use recovery::RecoveryKey;
pub use vault_key::{unwrap_key, wrap_key, Kek, VaultKey};

/// Version of the on-disk vault format produced by this crate.
pub const VAULT_FORMAT_VERSION: u32 = 1;
