use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{
    crypto::{
        b64, derive_kek, env_aad, open, random_bytes, seal, unwrap_key, wrap_key, KdfParams,
        RecoveryKey, SealedBlob, VaultKey, VAULT_FORMAT_VERSION,
    },
    domain::{check_password, validate_slug},
    error::AppError,
};

pub const DEFAULT_ENVIRONMENTS: &[&str] = &["development", "staging", "production"];

const KEY_CHECK_AAD: &[u8] = b"gitvault:v2:key-check";
const KEY_CHECK_VALUE: &[u8] = b"gitvault:v2:valid";

/// Returns current UNIX epoch seconds.
#[must_use]
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Plaintext header stored in `<store>/vault.json`. Contains no secrets or raw keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultHeader {
    pub format: u32,
    pub created_at: u64,
    pub kdf: KdfParams,
    #[serde(with = "b64")]
    pub password_salt: Vec<u8>,
    #[serde(with = "b64")]
    pub recovery_salt: Vec<u8>,
    pub wrapped_by_password: SealedBlob,
    pub wrapped_by_recovery: SealedBlob,
    #[serde(default)]
    pub key_check: Option<SealedBlob>,
}

impl VaultHeader {
    pub fn create(
        password: &str,
        kdf: KdfParams,
        now: u64,
    ) -> Result<(Self, VaultKey, RecoveryKey), AppError> {
        check_password(password)?;
        let key = VaultKey::generate();
        let recovery_key = RecoveryKey::generate();
        let password_salt = random_bytes::<16>().to_vec();
        let recovery_salt = random_bytes::<16>().to_vec();
        let wrapped_by_password = wrap_key(&derive_kek(password, &password_salt, &kdf)?, &key)?;
        let wrapped_by_recovery = wrap_key(&recovery_key.derive_kek(&recovery_salt)?, &key)?;
        let key_check = Some(seal(key.as_bytes(), KEY_CHECK_VALUE, KEY_CHECK_AAD)?);
        let header = Self {
            format: VAULT_FORMAT_VERSION,
            created_at: now,
            kdf,
            password_salt,
            recovery_salt,
            wrapped_by_password,
            wrapped_by_recovery,
            key_check,
        };
        Ok((header, key, recovery_key))
    }

    #[must_use]
    pub fn verify_key(&self, key: &VaultKey) -> bool {
        let Some(check) = self.key_check.as_ref() else {
            return true;
        };
        open(key.as_bytes(), check, KEY_CHECK_AAD)
            .is_ok_and(|plain| plain.as_slice() == KEY_CHECK_VALUE)
    }

    pub fn unlock_with_password(&self, password: &str) -> Result<VaultKey, AppError> {
        let kek = derive_kek(password, &self.password_salt, &self.kdf)?;
        unwrap_key(&kek, &self.wrapped_by_password)
    }

    pub fn unlock_with_recovery(&self, recovery: &RecoveryKey) -> Result<VaultKey, AppError> {
        let kek = recovery.derive_kek(&self.recovery_salt)?;
        unwrap_key(&kek, &self.wrapped_by_recovery)
    }

    pub fn change_password(&mut self, key: &VaultKey, new_password: &str) -> Result<(), AppError> {
        check_password(new_password)?;
        let salt = random_bytes::<16>().to_vec();
        self.wrapped_by_password = wrap_key(&derive_kek(new_password, &salt, &self.kdf)?, key)?;
        self.password_salt = salt;
        Ok(())
    }
}

/// Per-project metadata saved in `<store>/projects/<project>/project.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub environments: Vec<String>,
    pub created_at: u64,
}

impl ProjectManifest {
    pub fn new(name: &str, now: u64) -> Result<Self, AppError> {
        Ok(Self {
            name: validate_slug(name, "project")?,
            environments: DEFAULT_ENVIRONMENTS
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            created_at: now,
        })
    }

    pub fn ensure_environment(&mut self, env: &str) -> Result<bool, AppError> {
        let valid = validate_slug(env, "environment")?;
        if self.environments.iter().any(|e| e == &valid) {
            return Ok(false);
        }
        self.environments.push(valid);
        Ok(true)
    }

    pub fn remove_environment(&mut self, env: &str) -> bool {
        let before = self.environments.len();
        self.environments.retain(|e| e != env);
        self.environments.len() != before
    }
}

/// Encrypts a project environment's variable map with AAD binding.
pub fn seal_env(
    key: &VaultKey,
    project: &str,
    env: &str,
    vars: &BTreeMap<String, String>,
) -> Result<SealedBlob, AppError> {
    let payload = Zeroizing::new(
        serde_json::to_vec(vars).map_err(|e| AppError::Crypto(format!("serialize failed: {e}")))?,
    );
    seal(key.as_bytes(), &payload, &env_aad(project, env))
}

/// Decrypts and authenticates a project environment's variable map.
pub fn open_env(
    key: &VaultKey,
    project: &str,
    env: &str,
    blob: &SealedBlob,
) -> Result<BTreeMap<String, String>, AppError> {
    let plain = open(key.as_bytes(), blob, &env_aad(project, env))?;
    serde_json::from_slice(&plain)
        .map_err(|e| AppError::Crypto(format!("invalid encrypted environment payload: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_header_key_check_detects_wrong_key() {
        let (header, good_key, _) =
            VaultHeader::create("super-secret-pw", KdfParams::for_tests(), 100).unwrap();
        let other_key = VaultKey::generate();
        assert!(header.verify_key(&good_key));
        assert!(!header.verify_key(&other_key));
    }
}
