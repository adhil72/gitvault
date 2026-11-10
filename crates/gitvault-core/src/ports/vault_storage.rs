use crate::{
    domain::{
        ids::{SecretId, VaultId},
        vault::VaultHeader,
    },
    error::CoreError,
};

/// Reads and writes the files of local vault clones (no encryption here).
pub trait VaultStorage: Send + Sync {
    /// Ids of all vaults that exist on this computer.
    ///
    /// # Errors
    /// Fails on a storage error.
    fn vault_ids(&self) -> Result<Vec<VaultId>, CoreError>;

    fn exists(&self, id: &VaultId) -> bool;

    /// Creates the (empty) vault directory.
    ///
    /// # Errors
    /// Fails if it already exists or cannot be created.
    fn create(&self, id: &VaultId) -> Result<(), CoreError>;

    /// # Errors
    /// [`CoreError::NotFound`] when the vault or header is missing.
    fn read_header(&self, id: &VaultId) -> Result<VaultHeader, CoreError>;

    /// # Errors
    /// Fails on a storage error.
    fn write_header(&self, id: &VaultId, header: &VaultHeader) -> Result<(), CoreError>;

    /// # Errors
    /// Fails on a storage error.
    fn secret_ids(&self, id: &VaultId) -> Result<Vec<SecretId>, CoreError>;

    /// # Errors
    /// Fails on a storage error. A missing secret is `Ok(None)`.
    fn read_secret(&self, id: &VaultId, secret: &SecretId) -> Result<Option<Vec<u8>>, CoreError>;

    /// # Errors
    /// Fails on a storage error.
    fn write_secret(&self, id: &VaultId, secret: &SecretId, bytes: &[u8]) -> Result<(), CoreError>;

    /// # Errors
    /// Fails on a storage error. Deleting a missing secret is not an error.
    fn delete_secret(&self, id: &VaultId, secret: &SecretId) -> Result<(), CoreError>;

    /// Renames a vault directory (used after cloning into a temporary name).
    ///
    /// # Errors
    /// Fails if the target exists.
    fn rename(&self, from: &VaultId, to: &VaultId) -> Result<(), CoreError>;

    /// Deletes the local clone. Does not touch GitHub.
    ///
    /// # Errors
    /// Fails on a storage error.
    fn remove(&self, id: &VaultId) -> Result<(), CoreError>;
}
