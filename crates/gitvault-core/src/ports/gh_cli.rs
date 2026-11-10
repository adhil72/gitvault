use crate::error::CoreError;

/// Reads the token of an already signed-in GitHub CLI (`gh`), so no OAuth app is needed.
pub trait GhCli: Send + Sync {
    /// The CLI's current token.
    ///
    /// # Errors
    /// [`CoreError::Validation`] when `gh` is missing or not signed in.
    fn token(&self) -> Result<String, CoreError>;
}
