//! Secrets: create, read, update, delete, search, history and transfer.

mod catalog;
mod crud;
mod history;
mod mapping;
mod query;
mod transfer;

use std::sync::Arc;

use gitvault_crypto::VaultKey;

use super::context::Context;
use crate::{
    domain::{
        ids::{SecretId, VaultId},
        secret::{codec, SecretPayload},
        vault::layout::secret_path,
    },
    error::CoreError,
};

pub(crate) type Loaded = Vec<(SecretId, SecretPayload)>;

pub struct SecretService {
    ctx: Arc<Context>,
}

impl SecretService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    /// Parses the vault id and fetches its unlocked key.
    fn open(&self, vault: &str) -> Result<(VaultId, VaultKey), CoreError> {
        let id = VaultId::parse(vault)?;
        let key = self.ctx.require_key(&id)?;
        Ok((id, key))
    }

    fn load(
        &self,
        vault: &VaultId,
        key: &VaultKey,
        secret: &SecretId,
    ) -> Result<SecretPayload, CoreError> {
        let bytes = self
            .ctx
            .storage
            .read_secret(vault, secret)?
            .ok_or(CoreError::NotFound { entity: "secret" })?;
        codec::decrypt(key, vault, secret, &bytes)
    }

    fn load_all(&self, vault: &VaultId, key: &VaultKey) -> Result<Loaded, CoreError> {
        let mut all = Vec::new();
        for secret in self.ctx.storage.secret_ids(vault)? {
            let payload = self.load(vault, key, &secret)?;
            all.push((secret, payload));
        }
        Ok(all)
    }

    /// Encrypts and writes a secret file without committing.
    fn write_file(
        &self,
        vault: &VaultId,
        key: &VaultKey,
        secret: &SecretId,
        payload: &SecretPayload,
    ) -> Result<(), CoreError> {
        let bytes = codec::encrypt(key, vault, secret, payload)?;
        self.ctx.storage.write_secret(vault, secret, &bytes)
    }

    /// Commits pending changes. Messages contain ids only, never names or values.
    fn commit(&self, vault: &VaultId, message: &str) -> Result<(), CoreError> {
        self.ctx
            .vcs
            .commit_all(vault, message, &self.ctx.author())
            .map(|_| ())
    }

    fn save(
        &self,
        vault: &VaultId,
        key: &VaultKey,
        secret: &SecretId,
        payload: &SecretPayload,
    ) -> Result<(), CoreError> {
        self.write_file(vault, key, secret, payload)
    }

    fn commit_secret(
        &self,
        vault: &VaultId,
        action: &str,
        secret: &SecretId,
    ) -> Result<(), CoreError> {
        self.commit(vault, &format!("secret: {action} {}", secret.short()))
    }

    /// Rejects a second secret with the same name in the same environment.
    fn ensure_unique(
        all: &Loaded,
        name: &str,
        environment: Option<&str>,
        except: Option<&SecretId>,
    ) -> Result<(), CoreError> {
        let clash = all.iter().any(|(id, p)| {
            Some(id) != except
                && p.name.eq_ignore_ascii_case(name)
                && p.environment.as_deref().map(str::to_lowercase)
                    == environment.map(str::to_lowercase)
        });
        if clash {
            return Err(CoreError::Conflict(format!(
                "a secret named {name} already exists in that environment"
            )));
        }
        Ok(())
    }

    fn path_of(secret: &SecretId) -> String {
        secret_path(secret)
    }
}
