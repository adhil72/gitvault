//! The plaintext vault header stored in `.gitvault/vault.json`.
//!
//! It holds the KDF settings and two wrapped copies of the vault key (one for
//! the master password, one for the recovery key). It contains no secrets.

use gitvault_crypto::{
    b64, derive_kek, random, unwrap_key, wrap_key, CryptoError, KdfParams, RecoveryKey, Sealed,
    VaultKey, VAULT_FORMAT_VERSION,
};
use serde::{Deserialize, Serialize};

use super::{DeviceKey, TrustedDevice, VaultName};
use crate::{domain::ids::VaultId, error::CoreError};

/// Shortest accepted master password.
pub const MIN_PASSWORD_LEN: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultHeader {
    pub format: u32,
    pub vault_id: VaultId,
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub kdf: KdfParams,
    #[serde(with = "b64")]
    pub password_salt: Vec<u8>,
    #[serde(with = "b64")]
    pub recovery_salt: Vec<u8>,
    pub wrapped_by_password: Sealed,
    pub wrapped_by_recovery: Sealed,
    /// Devices that unlock without the master password (see [`TrustedDevice`]).
    #[serde(default)]
    pub devices: Vec<TrustedDevice>,
}

/// Result of creating a vault: the header to store plus the one-time secrets.
pub struct NewVault {
    pub header: VaultHeader,
    pub key: VaultKey,
    pub recovery_key: RecoveryKey,
}

fn check_password(password: &str) -> Result<(), CoreError> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(CoreError::Validation(format!(
            "master password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

fn wrong_password(error: CryptoError) -> CoreError {
    match error {
        CryptoError::Decrypt => CoreError::WrongPassword,
        other => other.into(),
    }
}

impl VaultHeader {
    /// Creates a new vault header with a fresh vault key and recovery key.
    ///
    /// # Errors
    /// Fails for a short password or a cryptographic error.
    pub fn create(
        id: VaultId,
        name: &VaultName,
        description: &str,
        password: &str,
        kdf: KdfParams,
        now: i64,
    ) -> Result<NewVault, CoreError> {
        check_password(password)?;
        let key = VaultKey::generate();
        let recovery_key = RecoveryKey::generate();
        let password_salt = random::bytes::<16>().to_vec();
        let recovery_salt = random::bytes::<16>().to_vec();
        let wrapped_by_password = wrap_key(&derive_kek(password, &password_salt, &kdf)?, &key)?;
        let wrapped_by_recovery = wrap_key(&recovery_key.derive_kek(&recovery_salt)?, &key)?;
        let header = Self {
            format: VAULT_FORMAT_VERSION,
            vault_id: id,
            name: name.as_str().to_owned(),
            description: description.trim().to_owned(),
            created_at: now,
            kdf,
            password_salt,
            recovery_salt,
            wrapped_by_password,
            wrapped_by_recovery,
            devices: Vec::new(),
        };
        Ok(NewVault {
            header,
            key,
            recovery_key,
        })
    }

    /// # Errors
    /// [`CoreError::WrongPassword`] when the password is wrong.
    pub fn unlock_with_password(&self, password: &str) -> Result<VaultKey, CoreError> {
        let kek = derive_kek(password, &self.password_salt, &self.kdf)?;
        unwrap_key(&kek, &self.wrapped_by_password).map_err(wrong_password)
    }

    /// # Errors
    /// [`CoreError::WrongPassword`] when the recovery key is wrong.
    pub fn unlock_with_recovery(&self, recovery: &RecoveryKey) -> Result<VaultKey, CoreError> {
        let kek = recovery.derive_kek(&self.recovery_salt)?;
        unwrap_key(&kek, &self.wrapped_by_recovery).map_err(wrong_password)
    }

    /// Trusts a new device: stores the vault key wrapped by a fresh device secret.
    ///
    /// # Errors
    /// Fails on a cryptographic error.
    pub fn trust_device(
        &mut self,
        key: &VaultKey,
        name: &str,
        now: i64,
    ) -> Result<DeviceKey, CoreError> {
        let device = DeviceKey::generate();
        self.devices.push(TrustedDevice {
            id: device.id().to_owned(),
            name: name.trim().to_owned(),
            created_at: now,
            wrapped: device.wrap(key)?,
        });
        Ok(device)
    }

    /// # Errors
    /// [`CoreError::PermissionDenied`] when the device is not (or no longer) trusted.
    pub fn unlock_with_device(&self, device: &DeviceKey) -> Result<VaultKey, CoreError> {
        let entry = self
            .devices
            .iter()
            .find(|d| d.id == device.id())
            .ok_or_else(|| {
                CoreError::PermissionDenied("this device is not trusted for this vault".into())
            })?;
        device.unwrap(&entry.wrapped).map_err(wrong_password)
    }

    /// Removes a device. Returns whether it existed.
    pub fn revoke_device(&mut self, device_id: &str) -> bool {
        let before = self.devices.len();
        self.devices.retain(|d| d.id != device_id);
        self.devices.len() != before
    }

    /// Re-wraps the vault key under a new master password (secrets are untouched).
    ///
    /// # Errors
    /// Fails for a short password or a cryptographic error.
    pub fn change_password(&mut self, key: &VaultKey, new_password: &str) -> Result<(), CoreError> {
        check_password(new_password)?;
        let salt = random::bytes::<16>().to_vec();
        self.wrapped_by_password = wrap_key(&derive_kek(new_password, &salt, &self.kdf)?, key)?;
        self.password_salt = salt;
        Ok(())
    }

    /// Creates a new recovery key; the old one stops working.
    ///
    /// # Errors
    /// Fails on a cryptographic error.
    pub fn regenerate_recovery(&mut self, key: &VaultKey) -> Result<RecoveryKey, CoreError> {
        let recovery = RecoveryKey::generate();
        let salt = random::bytes::<16>().to_vec();
        self.wrapped_by_recovery = wrap_key(&recovery.derive_kek(&salt)?, key)?;
        self.recovery_salt = salt;
        Ok(recovery)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_vault(password: &str) -> NewVault {
        let name = VaultName::parse("Test").unwrap();
        VaultHeader::create(
            VaultId::parse("v1").unwrap(),
            &name,
            "",
            password,
            KdfParams::for_tests(),
            0,
        )
        .unwrap()
    }

    #[test]
    fn password_unlocks_and_wrong_password_is_rejected() {
        let nv = new_vault("correct horse");
        assert_eq!(
            nv.header
                .unlock_with_password("correct horse")
                .unwrap()
                .as_bytes(),
            nv.key.as_bytes()
        );
        assert!(matches!(
            nv.header.unlock_with_password("wrong"),
            Err(CoreError::WrongPassword)
        ));
    }

    #[test]
    fn recovery_key_unlocks() {
        let nv = new_vault("correct horse");
        let parsed = RecoveryKey::parse(&nv.recovery_key.to_display()).unwrap();
        assert_eq!(
            nv.header.unlock_with_recovery(&parsed).unwrap().as_bytes(),
            nv.key.as_bytes()
        );
        assert!(matches!(
            nv.header.unlock_with_recovery(&RecoveryKey::generate()),
            Err(CoreError::WrongPassword)
        ));
    }

    #[test]
    fn changing_password_keeps_the_same_vault_key() {
        let mut nv = new_vault("old password");
        nv.header.change_password(&nv.key, "new password").unwrap();
        assert!(nv.header.unlock_with_password("old password").is_err());
        assert_eq!(
            nv.header
                .unlock_with_password("new password")
                .unwrap()
                .as_bytes(),
            nv.key.as_bytes()
        );
    }

    #[test]
    fn regenerating_recovery_invalidates_the_old_key() {
        let mut nv = new_vault("some password");
        let new_key = nv.header.regenerate_recovery(&nv.key).unwrap();
        assert!(nv.header.unlock_with_recovery(&nv.recovery_key).is_err());
        assert!(nv.header.unlock_with_recovery(&new_key).is_ok());
    }

    #[test]
    fn trusted_device_unlocks_until_revoked() {
        let mut nv = new_vault("correct horse");
        let device = nv.header.trust_device(&nv.key, "laptop", 5).unwrap();
        let reloaded: VaultHeader =
            serde_json::from_str(&serde_json::to_string(&nv.header).unwrap()).unwrap();
        assert_eq!(
            reloaded.unlock_with_device(&device).unwrap().as_bytes(),
            nv.key.as_bytes()
        );
        assert!(!serde_json::to_string(&nv.header)
            .unwrap()
            .contains(&device.to_text()));
        assert!(nv.header.revoke_device(device.id()));
        assert!(matches!(
            nv.header.unlock_with_device(&device),
            Err(CoreError::PermissionDenied(_))
        ));
    }

    #[test]
    fn rejects_short_passwords() {
        let name = VaultName::parse("T").unwrap();
        let result = VaultHeader::create(
            VaultId::parse("v").unwrap(),
            &name,
            "",
            "short",
            KdfParams::for_tests(),
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn header_json_contains_no_key_material_in_plain_form() {
        let nv = new_vault("correct horse");
        let json = serde_json::to_string(&nv.header).unwrap();
        assert!(!json.contains(&nv.key.to_base64()));
    }
}
