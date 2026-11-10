//! Typed errors with stable codes shared by the desktop app and the CLI.

use gitvault_crypto::CryptoError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable, machine-readable error code. The CLI maps each code to an exit code
/// and the desktop app forwards it to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "E_INTERNAL")]
    Internal,
    #[serde(rename = "E_USAGE")]
    Usage,
    #[serde(rename = "E_NOT_AUTHENTICATED")]
    NotAuthenticated,
    #[serde(rename = "E_LOCKED")]
    VaultLocked,
    #[serde(rename = "E_NOT_FOUND")]
    NotFound,
    #[serde(rename = "E_CONFLICT")]
    Conflict,
    #[serde(rename = "E_NETWORK")]
    Network,
    #[serde(rename = "E_PERMISSION_DENIED")]
    PermissionDenied,
    #[serde(rename = "E_VALIDATION")]
    Validation,
    #[serde(rename = "E_WRONG_PASSWORD")]
    WrongPassword,
}

impl ErrorCode {
    /// Process exit code used by the CLI (see `docs/architecture.md` section 8.1).
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Internal => 1,
            Self::Usage => 2,
            Self::NotAuthenticated => 3,
            Self::VaultLocked => 4,
            Self::NotFound => 5,
            Self::Conflict => 6,
            Self::Network => 7,
            Self::PermissionDenied => 8,
            Self::Validation => 9,
            Self::WrongPassword => 10,
        }
    }

    /// The wire string, e.g. `E_LOCKED`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Internal => "E_INTERNAL",
            Self::Usage => "E_USAGE",
            Self::NotAuthenticated => "E_NOT_AUTHENTICATED",
            Self::VaultLocked => "E_LOCKED",
            Self::NotFound => "E_NOT_FOUND",
            Self::Conflict => "E_CONFLICT",
            Self::Network => "E_NETWORK",
            Self::PermissionDenied => "E_PERMISSION_DENIED",
            Self::Validation => "E_VALIDATION",
            Self::WrongPassword => "E_WRONG_PASSWORD",
        }
    }
}

/// Every failure a service can return.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("not signed in to GitHub")]
    NotAuthenticated,
    #[error("the vault is locked")]
    VaultLocked,
    #[error("wrong master password or recovery key")]
    WrongPassword,
    #[error("{entity} not found")]
    NotFound { entity: &'static str },
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("invalid input: {0}")]
    Validation(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl CoreError {
    /// The stable code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::NotAuthenticated => ErrorCode::NotAuthenticated,
            Self::VaultLocked => ErrorCode::VaultLocked,
            Self::WrongPassword => ErrorCode::WrongPassword,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::Conflict(_) => ErrorCode::Conflict,
            Self::Network(_) => ErrorCode::Network,
            Self::PermissionDenied(_) => ErrorCode::PermissionDenied,
            Self::Validation(_) => ErrorCode::Validation,
            Self::Storage(_) | Self::Git(_) | Self::Internal(_) => ErrorCode::Internal,
        }
    }

    /// A short hint telling the user (or an AI tool) what to do next.
    #[must_use]
    pub const fn hint(&self) -> Option<&'static str> {
        match self {
            Self::NotAuthenticated => Some("gitvault auth login"),
            Self::VaultLocked => Some("gitvault unlock"),
            Self::WrongPassword => Some("check the master password or use the recovery key"),
            _ => None,
        }
    }
}

impl From<CryptoError> for CoreError {
    fn from(error: CryptoError) -> Self {
        Self::Internal(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreError, ErrorCode};

    #[test]
    fn locked_vault_maps_to_exit_code_four() {
        assert_eq!(CoreError::VaultLocked.code(), ErrorCode::VaultLocked);
        assert_eq!(ErrorCode::VaultLocked.exit_code(), 4);
    }

    #[test]
    fn every_code_has_a_distinct_exit_code() {
        let codes = [
            ErrorCode::Internal,
            ErrorCode::Usage,
            ErrorCode::NotAuthenticated,
            ErrorCode::VaultLocked,
            ErrorCode::NotFound,
            ErrorCode::Conflict,
            ErrorCode::Network,
            ErrorCode::PermissionDenied,
            ErrorCode::Validation,
            ErrorCode::WrongPassword,
        ];
        let mut seen: Vec<u8> = codes.iter().map(|c| c.exit_code()).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), codes.len());
    }
}
