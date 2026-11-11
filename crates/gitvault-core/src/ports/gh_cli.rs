use crate::error::CoreError;

/// One account signed in to the GitHub CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhCliAccount {
    pub login: String,
    /// The account `gh` itself currently uses.
    pub active: bool,
}

/// Reads tokens of an already signed-in GitHub CLI (`gh`), so no OAuth app is needed.
pub trait GhCli: Send + Sync {
    /// The token of the CLI's active account.
    ///
    /// # Errors
    /// [`CoreError::Validation`] when `gh` is missing or not signed in.
    fn token(&self) -> Result<String, CoreError>;

    /// Every `github.com` account known to the CLI (empty when `gh` is missing or signed out).
    fn accounts(&self) -> Vec<GhCliAccount>;

    /// The token of one specific account, without changing which account `gh` has active.
    ///
    /// # Errors
    /// [`CoreError::Validation`] when `gh` is missing or has no such account.
    fn token_for(&self, login: &str) -> Result<String, CoreError>;
}
