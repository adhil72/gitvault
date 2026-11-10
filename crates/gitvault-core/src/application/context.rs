//! Shared dependencies of all services.

use std::sync::Arc;

use gitvault_crypto::{KdfParams, VaultKey};

use super::{keys, session::SessionManager};
use crate::{
    domain::{
        account::GithubAccount,
        audit::{AuditEvent, AuditKind},
        ids::{SecretId, VaultId},
        settings::Settings,
    },
    error::CoreError,
    ports::{
        AuditSink, Author, Clock, GhCli, GitHost, IdGenerator, KeyValueStore, VaultStorage,
        VersionControl,
    },
};

/// Static configuration chosen by the front door (CLI or desktop app).
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Name shown in the audit log, e.g. `Linux · CLI`.
    pub device_name: String,
    /// Default GitHub OAuth app client id (settings can override it).
    pub github_client_id: Option<String>,
    pub kdf: KdfParams,
}

/// All ports plus configuration. Cheap to share through an `Arc`.
pub struct Context {
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub kv: Arc<dyn KeyValueStore>,
    pub storage: Arc<dyn VaultStorage>,
    pub vcs: Arc<dyn VersionControl>,
    pub host: Arc<dyn GitHost>,
    pub gh_cli: Arc<dyn GhCli>,
    pub audit: Arc<dyn AuditSink>,
    pub sessions: SessionManager,
    pub config: CoreConfig,
}

impl Context {
    #[must_use]
    pub fn now(&self) -> i64 {
        self.clock.now_unix_secs()
    }

    /// Current settings (defaults when none are stored or they are unreadable).
    #[must_use]
    pub fn settings(&self) -> Settings {
        self.kv
            .get(keys::SETTINGS)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    fn idle_secs(&self) -> i64 {
        i64::from(self.settings().auto_lock_minutes) * 60
    }

    /// The GitHub token, or [`CoreError::NotAuthenticated`].
    ///
    /// # Errors
    /// [`CoreError::NotAuthenticated`] when not signed in.
    pub fn token(&self) -> Result<String, CoreError> {
        self.kv
            .get(keys::GITHUB_TOKEN)?
            .ok_or(CoreError::NotAuthenticated)
    }

    #[must_use]
    pub fn account(&self) -> Option<GithubAccount> {
        let json = self.kv.get(keys::GITHUB_ACCOUNT).ok().flatten()?;
        serde_json::from_str(&json).ok()
    }

    /// Commit author: the GitHub account when signed in, a local identity otherwise.
    #[must_use]
    pub fn author(&self) -> Author {
        match self.account() {
            Some(account) => Author {
                name: account.login.clone(),
                email: account.commit_email(),
            },
            None => Author {
                name: "GitVault".into(),
                email: "gitvault@localhost".into(),
            },
        }
    }

    /// The unlocked key of a vault.
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] when locked or idle for too long.
    pub fn require_key(&self, vault: &VaultId) -> Result<VaultKey, CoreError> {
        self.sessions.get(vault, self.idle_secs())
    }

    #[must_use]
    pub fn is_unlocked(&self, vault: &VaultId) -> bool {
        self.sessions.is_unlocked(vault, self.idle_secs())
    }

    /// Records an audit event. Failures are ignored: auditing must never block an action.
    pub fn record(
        &self,
        vault: &VaultId,
        kind: AuditKind,
        secret: Option<(&SecretId, &str)>,
        detail: Option<&str>,
    ) {
        let event = AuditEvent {
            at: self.now(),
            kind,
            vault_id: vault.to_string(),
            secret_id: secret.map(|(id, _)| id.to_string()),
            secret_name: secret.map(|(_, name)| name.to_owned()),
            device: self.config.device_name.clone(),
            detail: detail.map(str::to_owned),
        };
        let _ = self.audit.append(&event);
    }
}
