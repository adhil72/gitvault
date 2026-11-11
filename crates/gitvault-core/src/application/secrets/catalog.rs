//! A device-local index of secret names so a locked vault can still be browsed.
//!
//! It holds everything except values and notes and lives only in this device's private
//! store: it is never committed, synced or put in the GitHub repository, so the vault
//! format and its "names are encrypted" guarantee are unchanged. It is refreshed every
//! time the vault is read while unlocked.

use serde::{Deserialize, Serialize};

use super::{Loaded, SecretService};
use crate::{
    application::keys,
    domain::{
        ids::{SecretId, VaultId},
        secret::{SecretPayload, SecretValue},
    },
    error::CoreError,
};

#[derive(Serialize, Deserialize)]
struct Entry {
    id: String,
    payload: SecretPayload,
}

fn redact(payload: &SecretPayload) -> SecretPayload {
    SecretPayload {
        value: SecretValue::new(""),
        notes: String::new(),
        ..payload.clone()
    }
}

impl SecretService {
    /// Stores the names of an unlocked vault for later locked browsing.
    pub(super) fn save_catalog(&self, vault: &VaultId, all: &Loaded) -> Result<(), CoreError> {
        let entries: Vec<Entry> = all
            .iter()
            .map(|(id, payload)| Entry {
                id: id.to_string(),
                payload: redact(payload),
            })
            .collect();
        let json =
            serde_json::to_string(&entries).map_err(|e| CoreError::Internal(e.to_string()))?;
        let key = keys::catalog(vault.as_str());
        // Skip the write when nothing changed: lists are re-read often.
        if self.ctx.kv.get(&key)?.as_deref() == Some(json.as_str()) {
            return Ok(());
        }
        self.ctx.kv.set(&key, &json)
    }

    /// The stored names of a vault, or [`CoreError::VaultLocked`] when this device never unlocked it.
    pub(super) fn read_catalog(&self, vault: &VaultId) -> Result<Loaded, CoreError> {
        let json = self
            .ctx
            .kv
            .get(&keys::catalog(vault.as_str()))?
            .ok_or(CoreError::VaultLocked)?;
        let entries: Vec<Entry> =
            serde_json::from_str(&json).map_err(|e| CoreError::Storage(e.to_string()))?;
        entries
            .into_iter()
            .map(|e| Ok((SecretId::parse(&e.id)?, e.payload)))
            .collect()
    }

    /// Everything readable without a password: the live data when unlocked (refreshing the
    /// index), the stored index otherwise.
    pub(super) fn browse(&self, vault_id: &str) -> Result<(VaultId, Loaded), CoreError> {
        let vault = VaultId::parse(vault_id)?;
        match self.ctx.require_key(&vault) {
            Ok(key) => {
                let all = self.load_all(&vault, &key)?;
                self.save_catalog(&vault, &all)?;
                Ok((vault, all))
            }
            Err(CoreError::VaultLocked) => {
                let all = self.read_catalog(&vault)?;
                Ok((vault, all))
            }
            Err(error) => Err(error),
        }
    }
}
