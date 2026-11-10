//! Unlocked vault keys.
//!
//! Keys live in memory. When `persist` is on (the CLI), they are also stored in
//! the private key-value store so the next command can reuse them until the
//! idle timeout; the desktop app keeps them in memory only.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use gitvault_crypto::VaultKey;
use serde::{Deserialize, Serialize};

use super::keys;
use crate::{
    domain::ids::VaultId,
    error::CoreError,
    ports::{Clock, KeyValueStore},
};

const TOKEN_PREFIX: &str = "gv1";

struct Entry {
    key: VaultKey,
    last_used: i64,
}

#[derive(Serialize, Deserialize)]
struct Stored {
    key: String,
    last_used: i64,
}

pub struct SessionManager {
    kv: Arc<dyn KeyValueStore>,
    clock: Arc<dyn Clock>,
    persist: bool,
    memory: Mutex<HashMap<String, Entry>>,
}

impl SessionManager {
    #[must_use]
    pub fn new(kv: Arc<dyn KeyValueStore>, clock: Arc<dyn Clock>, persist: bool) -> Self {
        Self {
            kv,
            clock,
            persist,
            memory: Mutex::new(HashMap::new()),
        }
    }

    fn entries(&self) -> Result<MutexGuard<'_, HashMap<String, Entry>>, CoreError> {
        self.memory
            .lock()
            .map_err(|_| CoreError::Internal("session lock poisoned".into()))
    }

    /// Stores an unlocked key.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn put(&self, vault: &VaultId, key: VaultKey) -> Result<(), CoreError> {
        let now = self.clock.now_unix_secs();
        if self.persist {
            let stored = Stored {
                key: key.to_base64(),
                last_used: now,
            };
            let json =
                serde_json::to_string(&stored).map_err(|e| CoreError::Internal(e.to_string()))?;
            self.kv.set(&keys::session(vault.as_str()), &json)?;
        }
        self.entries()?.insert(
            vault.to_string(),
            Entry {
                key,
                last_used: now,
            },
        );
        Ok(())
    }

    /// Returns the key if the vault is unlocked and not idle for more than
    /// `idle_secs` (0 = never expires). Refreshes the idle timer.
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] when locked or expired.
    pub fn get(&self, vault: &VaultId, idle_secs: i64) -> Result<VaultKey, CoreError> {
        let now = self.clock.now_unix_secs();
        self.load_persisted_if_needed(vault)?;
        let mut entries = self.entries()?;
        let entry = entries
            .get_mut(vault.as_str())
            .ok_or(CoreError::VaultLocked)?;
        if idle_secs > 0 && now - entry.last_used > idle_secs {
            entries.remove(vault.as_str());
            drop(entries);
            self.forget_persisted(vault)?;
            return Err(CoreError::VaultLocked);
        }
        entry.last_used = now;
        let key = entry.key.clone();
        drop(entries);
        self.touch_persisted(vault, &key, now)?;
        Ok(key)
    }

    #[must_use]
    pub fn is_unlocked(&self, vault: &VaultId, idle_secs: i64) -> bool {
        self.peek(vault, idle_secs)
    }

    /// Locks a vault and wipes its key from memory and the store.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn remove(&self, vault: &VaultId) -> Result<(), CoreError> {
        self.entries()?.remove(vault.as_str());
        self.forget_persisted(vault)
    }

    /// A portable token (`GITVAULT_SESSION`) for scripts and AI tools.
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] when the vault is locked.
    pub fn export_token(&self, vault: &VaultId, idle_secs: i64) -> Result<String, CoreError> {
        let key = self.get(vault, idle_secs)?;
        Ok(format!("{TOKEN_PREFIX}:{vault}:{}", key.to_base64()))
    }

    /// Accepts a token produced by [`SessionManager::export_token`].
    ///
    /// # Errors
    /// Validation error for malformed tokens.
    pub fn import_token(&self, token: &str) -> Result<VaultId, CoreError> {
        let mut parts = token.trim().splitn(3, ':');
        let (Some(prefix), Some(vault), Some(key)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(CoreError::Validation("malformed session token".into()));
        };
        if prefix != TOKEN_PREFIX {
            return Err(CoreError::Validation("unsupported session token".into()));
        }
        let vault = VaultId::parse(vault)?;
        self.put(&vault, VaultKey::from_base64(key)?)?;
        Ok(vault)
    }

    fn peek(&self, vault: &VaultId, idle_secs: i64) -> bool {
        self.get(vault, idle_secs).is_ok()
    }

    fn load_persisted_if_needed(&self, vault: &VaultId) -> Result<(), CoreError> {
        if !self.persist || self.entries()?.contains_key(vault.as_str()) {
            return Ok(());
        }
        let Some(json) = self.kv.get(&keys::session(vault.as_str()))? else {
            return Ok(());
        };
        let Ok(stored) = serde_json::from_str::<Stored>(&json) else {
            return Ok(());
        };
        let key = VaultKey::from_base64(&stored.key)?;
        self.entries()?.insert(
            vault.to_string(),
            Entry {
                key,
                last_used: stored.last_used,
            },
        );
        Ok(())
    }

    fn touch_persisted(&self, vault: &VaultId, key: &VaultKey, now: i64) -> Result<(), CoreError> {
        if !self.persist {
            return Ok(());
        }
        let stored = Stored {
            key: key.to_base64(),
            last_used: now,
        };
        let json =
            serde_json::to_string(&stored).map_err(|e| CoreError::Internal(e.to_string()))?;
        self.kv.set(&keys::session(vault.as_str()), &json)
    }

    fn forget_persisted(&self, vault: &VaultId) -> Result<(), CoreError> {
        if self.persist {
            self.kv.delete(&keys::session(vault.as_str()))?;
        }
        Ok(())
    }
}
