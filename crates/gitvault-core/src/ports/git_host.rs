use crate::{domain::account::GithubAccount, error::CoreError};

/// Response of GitHub's device-flow start.
#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePoll {
    Pending,
    SlowDown,
    Authorized { token: String },
    Expired,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRepo {
    /// `owner/name`
    pub full_name: String,
    pub clone_url: String,
    pub private: bool,
    pub updated_at: String,
}

/// What the signed-in GitHub user may do in a vault repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoAccess {
    /// No access, or the repository is invisible to this user.
    None,
    Read,
    Write,
}

/// GitHub: identity and repositories.
pub trait GitHost: Send + Sync {
    /// # Errors
    /// [`CoreError::Network`] on failure.
    fn device_code(&self, client_id: &str) -> Result<DeviceCode, CoreError>;

    /// # Errors
    /// [`CoreError::Network`] on failure.
    fn poll_device_token(
        &self,
        client_id: &str,
        device_code: &str,
    ) -> Result<DevicePoll, CoreError>;

    /// # Errors
    /// [`CoreError::NotAuthenticated`] for a rejected token.
    fn current_user(&self, token: &str) -> Result<GithubAccount, CoreError>;

    /// Repositories of the user that are GitVault vaults (topic `gitvault-vault`).
    ///
    /// # Errors
    /// [`CoreError::Network`] on failure.
    fn find_vault_repos(&self, token: &str) -> Result<Vec<RemoteRepo>, CoreError>;

    /// The user's permission on `owner/name`. This is what decides who may open a vault.
    ///
    /// # Errors
    /// [`CoreError::Network`] when GitHub cannot be reached.
    fn repo_access(&self, token: &str, full_name: &str) -> Result<RepoAccess, CoreError>;

    /// Creates a **private** repository tagged `gitvault-vault`.
    ///
    /// # Errors
    /// [`CoreError::Conflict`] if the name is taken, [`CoreError::Network`] otherwise.
    fn create_repo(
        &self,
        token: &str,
        name: &str,
        description: &str,
    ) -> Result<RemoteRepo, CoreError>;
}
