//! Vaults: list, create, connect, unlock, lock and key management.

use std::{collections::HashSet, sync::Arc};

use gitvault_crypto::RecoveryKey;

use super::{context::Context, keys};
use crate::ports::RepoAccess;
use crate::{
    domain::{
        audit::AuditKind,
        ids::VaultId,
        vault::{VaultHeader, VaultName},
    },
    dto::{
        CreateVaultInput, CreatedVault, RemoteStatus, UnlockInput, VaultList, VaultState,
        VaultSummary,
    },
    error::CoreError,
};

pub struct VaultService {
    pub(super) ctx: Arc<Context>,
}

impl VaultService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    /// The vaults cloned on this device. Reads only local files, so it is instant and works
    /// offline; use [`Self::discover`] to find vaults that are not cloned yet.
    ///
    /// # Errors
    /// Fails on a local storage error.
    pub fn list(&self) -> Result<VaultList, CoreError> {
        let (vaults, _) = self.local_vaults()?;
        Ok(VaultList {
            vaults,
            remote_error: None,
        })
    }

    /// Vaults on GitHub that are not cloned on this device yet (contacts GitHub).
    ///
    /// # Errors
    /// Fails on a local storage error (GitHub problems go into `remote_error`).
    pub fn discover(&self) -> Result<VaultList, CoreError> {
        let (_, known_repos) = self.local_vaults()?;
        let mut vaults = Vec::new();
        let remote_error = self.add_remote_only(&mut vaults, &known_repos);
        Ok(VaultList {
            vaults,
            remote_error,
        })
    }

    fn local_vaults(&self) -> Result<(Vec<VaultSummary>, HashSet<String>), CoreError> {
        let current = self.current();
        let mut vaults = Vec::new();
        let mut known_repos = HashSet::new();
        for id in self.ctx.storage.vault_ids()? {
            let Ok(summary) = self.summary(&id, current.as_deref()) else {
                continue;
            };
            if let Some(repo) = &summary.repo {
                known_repos.insert(repo.to_lowercase());
            }
            vaults.push(summary);
        }
        Ok((vaults, known_repos))
    }

    fn add_remote_only(
        &self,
        vaults: &mut Vec<VaultSummary>,
        known: &HashSet<String>,
    ) -> Option<String> {
        let token = self.ctx.token().ok()?;
        match self.ctx.host.find_vault_repos(&token) {
            Ok(repos) => {
                for repo in repos
                    .into_iter()
                    .filter(|r| !known.contains(&r.full_name.to_lowercase()))
                {
                    vaults.push(remote_only_summary(&repo.full_name));
                }
                None
            }
            Err(error) => Some(error.to_string()),
        }
    }

    /// One vault by id.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] when it does not exist locally.
    pub fn info(&self, vault_id: &str) -> Result<VaultSummary, CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.summary(&id, self.current().as_deref())
    }

    /// Creates a vault, optionally with a private GitHub repository.
    ///
    /// # Errors
    /// Validation errors, [`CoreError::NotAuthenticated`] when a remote is requested
    /// without signing in, and storage/git errors. A failure to push to GitHub is
    /// reported in `remote_error` and does not undo the local vault.
    pub fn create(&self, input: &CreateVaultInput) -> Result<CreatedVault, CoreError> {
        let name = VaultName::parse(&input.name)?;
        let token = if input.create_remote {
            Some(self.ctx.token()?)
        } else {
            None
        };
        let id = VaultId::parse(&self.ctx.ids.new_id())?;
        let description = input.description.as_deref().unwrap_or_default();
        let created = VaultHeader::create(
            id.clone(),
            &name,
            description,
            &input.master_password,
            self.ctx.config.kdf.clone(),
            self.ctx.now(),
        )?;
        self.write_initial(&id, &created.header)?;
        self.ctx.sessions.put(&id, created.key.clone())?;
        let remote_error = token
            .and_then(|t| self.publish(&id, input, &t).err())
            .map(|e| e.to_string());
        self.set_current(&id)?;
        self.ctx
            .record(&id, AuditKind::Created, None, Some("vault"));
        Ok(CreatedVault {
            vault: self.summary(&id, Some(id.as_str()))?,
            recovery_key: created.recovery_key.to_display(),
            remote_error,
        })
    }

    fn write_initial(&self, id: &VaultId, header: &VaultHeader) -> Result<(), CoreError> {
        self.ctx.storage.create(id)?;
        let result = self
            .ctx
            .storage
            .write_header(id, header)
            .and_then(|()| self.ctx.vcs.init(id))
            .and_then(|()| {
                self.ctx
                    .vcs
                    .commit_all(id, "vault: create", &self.ctx.author())
                    .map(|_| ())
            });
        if result.is_err() {
            let _ = self.ctx.storage.remove(id);
        }
        result
    }

    fn publish(
        &self,
        id: &VaultId,
        input: &CreateVaultInput,
        token: &str,
    ) -> Result<(), CoreError> {
        let repo_name = input
            .repo_name
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map_or_else(
                || slugify(&format!("gitvault-{}", input.name)),
                str::to_owned,
            );
        let description = input
            .description
            .as_deref()
            .unwrap_or("GitVault encrypted vault");
        self.push_to_new_repo(id, &repo_name, description, token)
    }

    fn push_to_new_repo(
        &self,
        id: &VaultId,
        repo_name: &str,
        description: &str,
        token: &str,
    ) -> Result<(), CoreError> {
        let repo = self.ctx.host.create_repo(token, repo_name, description)?;
        self.ctx.vcs.set_remote(id, &repo.clone_url)?;
        self.ctx
            .vcs
            .sync(id, Some(token), &self.ctx.author())
            .map(|_| ())
    }

    /// Whether the vault's GitHub repository is still reachable (contacts GitHub).
    ///
    /// GitHub answers 404 both for a deleted repository and for a private one the user may
    /// not see. The two are told apart only when the repository belongs to the signed-in
    /// account, because an owner can always see their own repositories.
    ///
    /// # Errors
    /// [`CoreError::NotAuthenticated`] when not signed in, [`CoreError::NotFound`] for an
    /// unknown vault.
    pub fn remote_status(&self, vault_id: &str) -> Result<RemoteStatus, CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.ctx.storage.read_header(&id)?;
        let Some(repo) = self.remote_repo(&id) else {
            return Ok(RemoteStatus::LocalOnly);
        };
        match self.ctx.host.repo_access(&self.ctx.token()?, &repo) {
            Ok(RepoAccess::None) if self.owns(&repo) => Ok(RemoteStatus::Missing),
            Ok(RepoAccess::None) => Ok(RemoteStatus::NoAccess),
            Ok(_) => Ok(RemoteStatus::Reachable),
            Err(CoreError::Network(_)) => Ok(RemoteStatus::Offline),
            Err(other) => Err(other),
        }
    }

    /// Creates a new private repository for a vault whose repository was deleted, and pushes
    /// the local copy to it. The new repository keeps the old name when it is free.
    ///
    /// # Errors
    /// [`CoreError::Conflict`] when the old repository still exists or the name is taken.
    pub fn republish(&self, vault_id: &str) -> Result<VaultSummary, CoreError> {
        let id = VaultId::parse(vault_id)?;
        if !matches!(self.remote_status(vault_id)?, RemoteStatus::Missing) {
            return Err(CoreError::Conflict(
                "the GitHub repository still exists".into(),
            ));
        }
        let token = self.ctx.token()?;
        let name = self
            .remote_repo(&id)
            .and_then(|repo| repo.rsplit('/').next().map(str::to_owned))
            .unwrap_or_default();
        let description = self.ctx.storage.read_header(&id)?.description;
        self.push_to_new_repo(&id, &name, &description, &token)?;
        self.summary(&id, self.current().as_deref())
    }

    fn remote_repo(&self, id: &VaultId) -> Option<String> {
        self.ctx
            .vcs
            .remote_url(id)
            .ok()
            .flatten()
            .and_then(|url| parse_repo_url(&url))
    }

    fn owns(&self, repo: &str) -> bool {
        let owner = repo.split('/').next().unwrap_or_default();
        self.ctx
            .account()
            .is_some_and(|account| account.login.eq_ignore_ascii_case(owner))
    }

    /// Clones a vault found on GitHub onto this computer.
    ///
    /// # Errors
    /// Not signed in, repository not found, not a GitVault repository, already connected.
    pub fn connect(&self, repo_full_name: &str) -> Result<VaultSummary, CoreError> {
        let token = self.ctx.token()?;
        let repo = self
            .ctx
            .host
            .find_vault_repos(&token)?
            .into_iter()
            .find(|r| r.full_name.eq_ignore_ascii_case(repo_full_name))
            .ok_or(CoreError::NotFound {
                entity: "repository",
            })?;
        let temp = VaultId::parse(&format!("tmp-{}", self.ctx.ids.new_id()))?;
        let header = self.clone_header(&temp, &repo.clone_url, &token)?;
        let id = header.vault_id;
        if self.ctx.storage.exists(&id) {
            let _ = self.ctx.storage.remove(&temp);
            return Err(CoreError::Conflict(
                "this vault is already connected".into(),
            ));
        }
        self.ctx.storage.rename(&temp, &id)?;
        self.summary(&id, self.current().as_deref())
    }

    fn clone_header(
        &self,
        temp: &VaultId,
        url: &str,
        token: &str,
    ) -> Result<VaultHeader, CoreError> {
        let outcome = self
            .ctx
            .vcs
            .clone_repo(temp, url, Some(token))
            .and_then(|()| self.ctx.storage.read_header(temp));
        outcome.map_err(|error| {
            let _ = self.ctx.storage.remove(temp);
            match error {
                CoreError::NotFound { .. } => {
                    CoreError::Validation("that repository is not a GitVault vault".into())
                }
                other => other,
            }
        })
    }

    /// Unlocks a vault with its master password or the recovery key.
    ///
    /// # Errors
    /// [`CoreError::WrongPassword`] for wrong credentials.
    pub fn unlock(&self, input: &UnlockInput) -> Result<VaultSummary, CoreError> {
        let id = VaultId::parse(&input.vault_id)?;
        self.require_repo_access(&id)?;
        let header = self.ctx.storage.read_header(&id)?;
        let key = match (&input.password, &input.recovery_key) {
            (Some(password), _) => header.unlock_with_password(password)?,
            (None, Some(recovery)) => {
                let parsed = RecoveryKey::parse(recovery).map_err(|_| CoreError::WrongPassword)?;
                header.unlock_with_recovery(&parsed)?
            }
            (None, None) => {
                return Err(CoreError::Validation(
                    "password or recovery key required".into(),
                ))
            }
        };
        self.ctx.sessions.put(&id, key)?;
        self.set_current(&id)?;
        self.ctx.record(&id, AuditKind::Unlocked, None, None);
        self.summary(&id, Some(id.as_str()))
    }

    /// Locks a vault: its key is wiped from memory.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn lock(&self, vault_id: &str) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.ctx.sessions.remove(&id)?;
        self.ctx.record(&id, AuditKind::Locked, None, None);
        Ok(())
    }

    /// Changes the master password (secrets are not re-encrypted).
    ///
    /// # Errors
    /// [`CoreError::WrongPassword`] when `current_password` is wrong.
    pub fn change_password(
        &self,
        vault_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        let mut header = self.ctx.storage.read_header(&id)?;
        let key = header.unlock_with_password(current_password)?;
        header.change_password(&key, new_password)?;
        self.ctx.storage.write_header(&id, &header)?;
        self.ctx
            .vcs
            .commit_all(&id, "vault: change password", &self.ctx.author())?;
        Ok(())
    }

    /// Creates a new recovery key (the old one stops working) and returns it once.
    ///
    /// # Errors
    /// [`CoreError::WrongPassword`] when `current_password` is wrong.
    pub fn regenerate_recovery_key(
        &self,
        vault_id: &str,
        current_password: &str,
    ) -> Result<String, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let mut header = self.ctx.storage.read_header(&id)?;
        let key = header.unlock_with_password(current_password)?;
        let recovery = header.regenerate_recovery(&key)?;
        self.ctx.storage.write_header(&id, &header)?;
        self.ctx
            .vcs
            .commit_all(&id, "vault: regenerate recovery key", &self.ctx.author())?;
        Ok(recovery.to_display())
    }

    /// Removes the local copy of a vault. The GitHub repository is not touched.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn forget(&self, vault_id: &str) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.ctx.sessions.remove(&id)?;
        self.ctx.storage.remove(&id)?;
        self.ctx.kv.delete(&keys::catalog(id.as_str()))?;
        if self.current().as_deref() == Some(id.as_str()) {
            self.ctx.kv.delete(keys::CURRENT_VAULT)?;
        }
        Ok(())
    }

    /// The vault commands use when none is named.
    #[must_use]
    pub fn current(&self) -> Option<String> {
        self.ctx.kv.get(keys::CURRENT_VAULT).ok().flatten()
    }

    /// Chooses the default vault.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] if the vault does not exist locally.
    pub fn use_vault(&self, vault_id: &str) -> Result<VaultSummary, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let summary = self.summary(&id, None)?;
        self.set_current(&id)?;
        Ok(VaultSummary {
            current: true,
            ..summary
        })
    }

    /// A portable `GITVAULT_SESSION` token for scripts and AI tools.
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] when the vault is locked.
    pub fn export_session(&self, vault_id: &str) -> Result<String, CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.ctx
            .sessions
            .export_token(&id, i64::from(self.ctx.settings().auto_lock_minutes) * 60)
    }

    /// Accepts a `GITVAULT_SESSION` token and returns the vault it unlocks.
    ///
    /// # Errors
    /// Validation error for malformed tokens.
    pub fn import_session(&self, token: &str) -> Result<String, CoreError> {
        self.ctx
            .sessions
            .import_token(token)
            .map(|id| id.to_string())
    }

    /// Checks with GitHub that the signed-in user may still open this vault's repository.
    ///
    /// GitHub's repository permissions are the access list: the project manager removes
    /// someone from the repo and they can no longer unlock the vault here. Offline the check
    /// is skipped (the master password is still required), local-only vaults have no repo,
    /// and an owner whose repository was deleted may still open their local copy.
    pub(super) fn require_repo_access(&self, id: &VaultId) -> Result<(), CoreError> {
        match self.remote_status(id.as_str())? {
            RemoteStatus::NoAccess => Err(CoreError::PermissionDenied(format!(
                "your GitHub account has no access to {}",
                self.remote_repo(id).unwrap_or_default()
            ))),
            _ => Ok(()),
        }
    }

    pub(super) fn set_current(&self, id: &VaultId) -> Result<(), CoreError> {
        self.ctx.kv.set(keys::CURRENT_VAULT, id.as_str())
    }

    fn summary(&self, id: &VaultId, current: Option<&str>) -> Result<VaultSummary, CoreError> {
        let header = self.ctx.storage.read_header(id)?;
        let repo = self.remote_repo(id);
        Ok(VaultSummary {
            id: Some(id.to_string()),
            name: header.name,
            description: header.description,
            repo,
            state: VaultState::Local,
            unlocked: self.ctx.is_unlocked(id),
            current: current == Some(id.as_str()),
        })
    }
}

fn remote_only_summary(full_name: &str) -> VaultSummary {
    VaultSummary {
        id: None,
        name: full_name.rsplit('/').next().unwrap_or(full_name).to_owned(),
        description: String::new(),
        repo: Some(full_name.to_owned()),
        state: VaultState::RemoteOnly,
        unlocked: false,
        current: false,
    }
}

/// Extracts `owner/name` from a GitHub remote URL (https or ssh form).
#[must_use]
pub fn parse_repo_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let tail = trimmed.rsplit_once("github.com").map(|(_, rest)| rest)?;
    let path = tail.trim_start_matches([':', '/']);
    (path.split('/').count() == 2 && !path.starts_with('/')).then(|| path.to_owned())
}

/// A GitHub-safe repository name derived from free text.
#[must_use]
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.to_lowercase().chars() {
        let mapped = if c.is_ascii_alphanumeric() { c } else { '-' };
        if mapped != '-' || !out.ends_with('-') {
            out.push(mapped);
        }
    }
    out.trim_matches('-').chars().take(80).collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_repo_url, slugify};

    #[test]
    fn parses_https_and_ssh_urls() {
        assert_eq!(
            parse_repo_url("https://github.com/octocat/vault.git").as_deref(),
            Some("octocat/vault")
        );
        assert_eq!(
            parse_repo_url("git@github.com:octocat/vault.git").as_deref(),
            Some("octocat/vault")
        );
        assert_eq!(parse_repo_url("/tmp/local/remote.git"), None);
    }

    #[test]
    fn slugifies_names() {
        assert_eq!(
            slugify("gitvault-My Team Secrets!"),
            "gitvault-my-team-secrets"
        );
        assert_eq!(slugify("  --a__b--  "), "a-b");
    }
}
