//! Sync with GitHub: pull, push and conflict handling.

use std::sync::Arc;

use super::{context::Context, keys};
use crate::{
    domain::{
        audit::AuditKind,
        ids::{SecretId, VaultId},
        secret::codec,
        vault::layout::{secret_path, SECRETS_DIR, SECRET_EXTENSION},
    },
    dto::{ConflictDto, ConflictKeep, SyncResult, SyncState, SyncStatus},
    error::CoreError,
    ports::ConflictChoice,
};

pub struct SyncService {
    ctx: Arc<Context>,
}

impl SyncService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    /// Local sync state (does not contact GitHub).
    ///
    /// # Errors
    /// Fails on a git error.
    pub fn status(&self, vault_id: &str) -> Result<SyncStatus, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let repo = self.ctx.vcs.status(&id)?;
        let state = if !repo.has_remote {
            SyncState::NoRemote
        } else if !repo.conflicts.is_empty() {
            SyncState::Conflict
        } else if repo.pending > 0 {
            SyncState::Pending
        } else {
            SyncState::Synced
        };
        let last_sync = self
            .ctx
            .kv
            .get(&keys::last_sync(id.as_str()))?
            .and_then(|v| v.parse().ok());
        Ok(SyncStatus {
            state,
            pending: repo.pending,
            conflicts: u32::try_from(repo.conflicts.len()).unwrap_or(u32::MAX),
            last_sync,
        })
    }

    /// Pulls remote changes, then pushes local ones.
    ///
    /// # Errors
    /// [`CoreError::Network`] when GitHub is unreachable.
    pub fn sync_now(&self, vault_id: &str) -> Result<SyncResult, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let token = self.ctx.token().ok();
        let report = self
            .ctx
            .vcs
            .sync(&id, token.as_deref(), &self.ctx.author())?;
        if !report.no_remote && report.conflicts.is_empty() {
            self.ctx
                .kv
                .set(&keys::last_sync(id.as_str()), &self.ctx.now().to_string())?;
            self.ctx.record(
                &id,
                AuditKind::Synced,
                None,
                Some(&format!(
                    "{} pulled, {} pushed",
                    report.pulled, report.pushed
                )),
            );
        }
        Ok(SyncResult {
            pulled: report.pulled,
            pushed: report.pushed,
            conflicts: u32::try_from(report.conflicts.len()).unwrap_or(u32::MAX),
            no_remote: report.no_remote,
        })
    }

    /// Secrets edited on two devices. Names are shown when the vault is unlocked.
    ///
    /// # Errors
    /// Fails on a git error.
    pub fn conflicts(&self, vault_id: &str) -> Result<Vec<ConflictDto>, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let paths = self.ctx.vcs.status(&id)?.conflicts;
        Ok(paths
            .iter()
            .filter_map(|path| secret_id_from_path(path))
            .map(|secret| ConflictDto {
                name: self.local_name(&id, &secret),
                secret_id: secret.to_string(),
            })
            .collect())
    }

    fn local_name(&self, vault: &VaultId, secret: &SecretId) -> Option<String> {
        let key = self.ctx.require_key(vault).ok()?;
        let bytes = self
            .ctx
            .vcs
            .read_at(vault, "HEAD", &secret_path(secret))
            .ok()
            .flatten()?;
        codec::decrypt(&key, vault, secret, &bytes)
            .ok()
            .map(|p| p.name)
    }

    /// Keeps one side of a conflicted secret.
    ///
    /// # Errors
    /// Fails on a git error.
    pub fn resolve(
        &self,
        vault_id: &str,
        secret_id: &str,
        keep: &ConflictKeep,
    ) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        let secret = SecretId::parse(secret_id)?;
        let choice = match keep {
            ConflictKeep::Local => ConflictChoice::Local,
            ConflictKeep::Remote => ConflictChoice::Remote,
        };
        self.ctx
            .vcs
            .resolve(&id, &secret_path(&secret), choice, &self.ctx.author())
    }
}

fn secret_id_from_path(path: &str) -> Option<SecretId> {
    let name = path.strip_prefix(&format!("{SECRETS_DIR}/"))?;
    let stem = name.strip_suffix(&format!(".{SECRET_EXTENSION}"))?;
    SecretId::parse(stem).ok()
}

#[cfg(test)]
mod tests {
    use super::secret_id_from_path;

    #[test]
    fn extracts_secret_ids_from_paths() {
        assert_eq!(
            secret_id_from_path("secrets/abc-1.enc")
                .map(|s| s.to_string())
                .as_deref(),
            Some("abc-1")
        );
        assert!(secret_id_from_path(".gitvault/vault.json").is_none());
        assert!(secret_id_from_path("secrets/a/b.enc").is_none());
    }
}
