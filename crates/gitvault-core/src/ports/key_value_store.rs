use crate::error::CoreError;

/// Small private state: GitHub token, settings, sessions, last-sync times.
///
/// Implementations must keep the data readable only by the current user.
pub trait KeyValueStore: Send + Sync {
    /// # Errors
    /// Fails on a storage error.
    fn get(&self, key: &str) -> Result<Option<String>, CoreError>;

    /// # Errors
    /// Fails on a storage error.
    fn set(&self, key: &str, value: &str) -> Result<(), CoreError>;

    /// # Errors
    /// Fails on a storage error. Deleting a missing key is not an error.
    fn delete(&self, key: &str) -> Result<(), CoreError>;
}
