//! Create, read, update, rotate and delete secrets.

use super::{
    mapping::{self, normalize, RawFields},
    SecretService,
};
use crate::{
    domain::{
        audit::AuditKind,
        ids::SecretId,
        secret::{SecretPayload, SecretValue},
    },
    dto::{AddSecretInput, RevealedSecret, SecretDetail, UpdateSecretInput},
    error::CoreError,
};

fn require_value(value: &str) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(CoreError::Validation("a secret needs a value".into()));
    }
    Ok(())
}

impl SecretService {
    /// Adds a secret to an unlocked vault.
    ///
    /// # Errors
    /// Locked vault, validation errors, duplicate name in the same environment.
    pub fn add(&self, input: &AddSecretInput) -> Result<SecretDetail, CoreError> {
        let (vault, key) = self.open(&input.vault_id)?;
        let fields = normalize(&RawFields {
            name: &input.name,
            environment: input.environment.as_deref(),
            tags: &input.tags,
            notes: input.notes.as_deref(),
        })?;
        require_value(&input.value)?;
        Self::ensure_unique(
            &self.load_all(&vault, &key)?,
            &fields.name,
            fields.environment.as_deref(),
            None,
        )?;
        let (id, now) = (SecretId::parse(&self.ctx.ids.new_id())?, self.ctx.now());
        let payload = SecretPayload {
            name: fields.name,
            kind: input.kind,
            environment: fields.environment,
            tags: fields.tags,
            value: SecretValue::new(input.value.clone()),
            notes: fields.notes,
            expires_at: input.expires_at,
            remind_before_days: input.remind_before_days,
            favorite: false,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.save(&vault, &key, &id, &payload)?;
        self.commit_secret(&vault, "add", &id)?;
        self.ctx
            .record(&vault, AuditKind::Created, Some((&id, &payload.name)), None);
        Ok(mapping::detail(&id, &payload, now))
    }

    /// Metadata of one secret (no value). Works while locked, minus the notes.
    ///
    /// # Errors
    /// Unknown secret, or a locked vault this device has never unlocked.
    pub fn get(&self, vault_id: &str, secret_id: &str) -> Result<SecretDetail, CoreError> {
        let id = SecretId::parse(secret_id)?;
        if let Ok((vault, key)) = self.open(vault_id) {
            let payload = self.load(&vault, &key, &id)?;
            return Ok(mapping::detail(&id, &payload, self.ctx.now()));
        }
        let (_, all) = self.browse(vault_id)?;
        let (_, payload) = all
            .iter()
            .find(|(found, _)| *found == id)
            .ok_or(CoreError::NotFound { entity: "secret" })?;
        Ok(mapping::detail(&id, payload, self.ctx.now()))
    }

    /// The plaintext value. Recorded in the audit log.
    ///
    /// # Errors
    /// Locked vault or unknown secret.
    pub fn reveal(&self, vault_id: &str, secret_id: &str) -> Result<RevealedSecret, CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        let payload = self.load(&vault, &key, &id)?;
        self.ctx
            .record(&vault, AuditKind::Viewed, Some((&id, &payload.name)), None);
        Ok(RevealedSecret {
            value: payload.value.expose().to_owned(),
        })
    }

    /// Records that a value was copied to the clipboard.
    ///
    /// # Errors
    /// Locked vault or unknown secret.
    pub fn mark_copied(&self, vault_id: &str, secret_id: &str) -> Result<(), CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        let payload = self.load(&vault, &key, &id)?;
        self.ctx
            .record(&vault, AuditKind::Copied, Some((&id, &payload.name)), None);
        Ok(())
    }

    /// Replaces the editable fields. The value is kept unless a new one is given.
    ///
    /// # Errors
    /// Locked vault, unknown secret, validation errors, duplicate names.
    pub fn update(&self, input: &UpdateSecretInput) -> Result<SecretDetail, CoreError> {
        let (vault, key) = self.open(&input.vault_id)?;
        let id = SecretId::parse(&input.secret_id)?;
        let mut payload = self.load(&vault, &key, &id)?;
        let fields = normalize(&RawFields {
            name: &input.name,
            environment: input.environment.as_deref(),
            tags: &input.tags,
            notes: input.notes.as_deref(),
        })?;
        Self::ensure_unique(
            &self.load_all(&vault, &key)?,
            &fields.name,
            fields.environment.as_deref(),
            Some(&id),
        )?;
        payload.name = fields.name;
        payload.kind = input.kind;
        payload.environment = fields.environment;
        payload.tags = fields.tags;
        payload.notes = fields.notes;
        payload.expires_at = input.expires_at;
        payload.remind_before_days = input.remind_before_days;
        payload.favorite = input.favorite;
        if let Some(value) = input.value.as_deref().filter(|v| !v.is_empty()) {
            payload.value = SecretValue::new(value);
        }
        self.finish_change(&vault, &key, &id, payload, "update", AuditKind::Updated)
    }

    /// Sets a new value (for example after rotating an API key).
    ///
    /// # Errors
    /// Locked vault, unknown secret, empty value.
    pub fn rotate(
        &self,
        vault_id: &str,
        secret_id: &str,
        new_value: &str,
    ) -> Result<SecretDetail, CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        require_value(new_value)?;
        let mut payload = self.load(&vault, &key, &id)?;
        payload.value = SecretValue::new(new_value);
        self.finish_change(&vault, &key, &id, payload, "rotate", AuditKind::Rotated)
    }

    #[allow(clippy::too_many_arguments)] // vault, key, id and the change description belong together
    fn finish_change(
        &self,
        vault: &crate::domain::ids::VaultId,
        key: &gitvault_crypto::VaultKey,
        id: &SecretId,
        mut payload: SecretPayload,
        action: &str,
        kind: AuditKind,
    ) -> Result<SecretDetail, CoreError> {
        let now = self.ctx.now();
        payload.version += 1;
        payload.updated_at = now;
        self.save(vault, key, id, &payload)?;
        self.commit_secret(vault, action, id)?;
        self.ctx
            .record(vault, kind, Some((id, &payload.name)), None);
        Ok(mapping::detail(id, &payload, now))
    }

    /// Deletes a secret. It can be restored from history.
    ///
    /// # Errors
    /// Locked vault or unknown secret.
    pub fn delete(&self, vault_id: &str, secret_id: &str) -> Result<(), CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let id = SecretId::parse(secret_id)?;
        let name = self.load(&vault, &key, &id)?.name;
        self.ctx.storage.delete_secret(&vault, &id)?;
        self.commit_secret(&vault, "delete", &id)?;
        self.ctx
            .record(&vault, AuditKind::Deleted, Some((&id, &name)), None);
        Ok(())
    }
}
