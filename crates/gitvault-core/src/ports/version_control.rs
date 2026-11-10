use crate::{domain::ids::VaultId, error::CoreError};

/// Who a commit is attributed to.
#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub at: i64,
}

/// Local repository state relative to its remote.
#[derive(Debug, Clone, Default)]
pub struct RepoStatus {
    pub has_remote: bool,
    /// Commits not yet pushed.
    pub pending: u32,
    /// Files with unresolved merge conflicts.
    pub conflicts: Vec<String>,
}

/// Outcome of a sync.
#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub pulled: u32,
    pub pushed: u32,
    pub conflicts: Vec<String>,
    pub no_remote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Local,
    Remote,
}

/// Git operations on a vault repository. `token` is a GitHub access token used
/// only for network operations.
pub trait VersionControl: Send + Sync {
    /// # Errors
    /// Fails if git is unavailable or the repository cannot be created.
    fn init(&self, id: &VaultId) -> Result<(), CoreError>;

    /// # Errors
    /// Fails if git is unavailable.
    fn set_remote(&self, id: &VaultId, url: &str) -> Result<(), CoreError>;

    /// # Errors
    /// Fails on a git error.
    fn remote_url(&self, id: &VaultId) -> Result<Option<String>, CoreError>;

    /// Clones `url` into the vault directory `id`.
    ///
    /// # Errors
    /// Fails on network or git errors.
    fn clone_repo(&self, id: &VaultId, url: &str, token: Option<&str>) -> Result<(), CoreError>;

    /// Stages everything and commits. Returns `None` when nothing changed.
    ///
    /// # Errors
    /// Fails on a git error.
    fn commit_all(
        &self,
        id: &VaultId,
        message: &str,
        author: &Author,
    ) -> Result<Option<CommitInfo>, CoreError>;

    /// Commits that touched `path`, newest first (includes deletions).
    ///
    /// # Errors
    /// Fails on a git error.
    fn history(&self, id: &VaultId, path: &str) -> Result<Vec<CommitInfo>, CoreError>;

    /// File content at a commit, or `None` if the file did not exist there.
    ///
    /// # Errors
    /// Fails on a git error.
    fn read_at(&self, id: &VaultId, commit: &str, path: &str)
        -> Result<Option<Vec<u8>>, CoreError>;

    /// # Errors
    /// Fails on a git error.
    fn status(&self, id: &VaultId) -> Result<RepoStatus, CoreError>;

    /// Fetch, merge and push.
    ///
    /// # Errors
    /// [`CoreError::Network`] when the remote is unreachable.
    fn sync(
        &self,
        id: &VaultId,
        token: Option<&str>,
        author: &Author,
    ) -> Result<SyncReport, CoreError>;

    /// Resolves one conflicted file and completes the merge when none remain.
    ///
    /// # Errors
    /// Fails on a git error.
    fn resolve(
        &self,
        id: &VaultId,
        path: &str,
        choice: ConflictChoice,
        author: &Author,
    ) -> Result<(), CoreError>;
}
