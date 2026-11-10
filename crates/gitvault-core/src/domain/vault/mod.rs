//! Vault-level domain types.

mod device;
mod header;
pub mod layout;
mod name;
mod trust_policy;

pub use device::{DeviceKey, TrustedDevice};
pub use header::{NewVault, VaultHeader, MIN_PASSWORD_LEN};
pub use name::VaultName;
pub use trust_policy::{FolderRule, TrustContext, TrustPolicy};
