//! Version history and restore, backed by the vault's git history.

use super::SecretService;
use crate::{
    domain::{audit::AuditKind, ids::SecretId, secret::codec},
    dto::{HistoryEntry, SecretDetail},
    error::CoreError,
};

impl SecretService {
    /// Every commit that touched the secret, newest first (version 1 = oldest).
    ///
    /// # Errors
    /// Locked vault or a git error.
    pub fn history(&self, vault_id: &str, secret_id: &str) -> Result<Vec<HistoryEntry>, CoreError> {
        let (vault, _) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        let commits = self.ctx.vcs.history(&vault, &Self::path_of(&id))?;
        let total = commits.len();
        Ok(commits
            .into_iter()
            .enumerate()
            .map(|(index, commit)| HistoryEntry {
                commit: commit.sha,
                version: u32::try_from(total - index).unwrap_or(u32::MAX),
                message: commit.message,
                author: commit.author,
                at: commit.at,
            })
            .collect())
    }

    /// Writes an old version back as a new version (works for deleted secrets too).
    ///
    /// # Errors
    /// Locked vault, unknown commit, or the old file does not exist at that commit.
    pub fn restore(
        &self,
        vault_id: &str,
        secret_id: &str,
        commit: &str,
    ) -> Result<SecretDetail, CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        let old = self
            .ctx
            .vcs
            .read_at(&vault, commit, &Self::path_of(&id))?
            .ok_or(CoreError::NotFound { entity: "version" })?;
        let mut payload = codec::decrypt(&key, &vault, &id, &old)?;
        let current = self
            .load(&vault, &key, &id)
            .map(|p| p.version)
            .unwrap_or_default();
        let others = self.load_all(&vault, &key)?;
        Self::ensure_unique(
            &others,
            &payload.name,
            payload.environment.as_deref(),
            Some(&id),
        )?;
        let now = self.ctx.now();
        payload.version = current + 1;
        payload.updated_at = now;
        self.save(&vault, &key, &id, &payload)?;
        self.commit_secret(&vault, "restore", &id)?;
        self.ctx.record(
            &vault,
            AuditKind::Restored,
            Some((&id, &payload.name)),
            Some(commit),
        );
        Ok(super::mapping::detail(&id, &payload, now))
    }
}
