use thiserror::Error;

/// Structured errors for GitVault v2 with deterministic process exit codes and actionable hints.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("required CLI tool '{tool}' is missing")]
    PrerequisiteMissing { tool: String, hint: String },

    #[error("GitHub CLI is not authenticated")]
    NotAuthenticated { hint: String },

    #[error("vault is locked")]
    VaultLocked { hint: String },

    #[error("wrong master password or recovery key")]
    WrongPassword,

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Validation(String),

    #[error("cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("git command failed: {0}")]
    Git(String),

    #[error("gh command failed: {0}")]
    Gh(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Returns a human-friendly recovery hint when available.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::PrerequisiteMissing { hint, .. }
            | Self::NotAuthenticated { hint }
            | Self::VaultLocked { hint } => Some(hint),
            Self::WrongPassword => {
                Some("Check your master password, or unlock with --recovery-key.")
            }
            _ => None,
        }
    }

    /// Stable non-zero exit code mapped from error category.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::PrerequisiteMissing { .. } => 2,
            Self::NotAuthenticated { .. } => 3,
            Self::VaultLocked { .. } => 4,
            Self::WrongPassword => 5,
            Self::NotFound(_) => 6,
            Self::Validation(_) => 7,
            Self::Crypto(_) => 8,
            Self::Git(_) => 9,
            Self::Gh(_) => 10,
        }
    }
}
