//! Trusted devices: unlock a vault without the master password.
//!
//! The device secret never leaves the device (or the CI secret store it was exported to); the
//! vault header only holds the vault key wrapped by it. Changing the master password does not
//! affect trusted devices; revoke them to cut their access.

use std::path::Path;

use super::{keys, vaults::VaultService};
use crate::{
    domain::{
        audit::AuditKind,
        ids::VaultId,
        vault::{DeviceKey, FolderRule, TrustContext, TrustPolicy, VaultHeader},
    },
    dto::{DeviceDto, FolderRuleDto, TrustedDeviceDto, VaultSummary},
    error::CoreError,
};

impl VaultService {
    /// Trusts a device for an unlocked vault. With `for_this_device` the secret is kept here;
    /// otherwise it is returned once, to be stored on another machine (`GITVAULT_DEVICE_KEY`).
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] unless the vault is unlocked (so the password was just used).
    pub fn trust_device(
        &self,
        vault_id: &str,
        name: &str,
        for_this_device: bool,
    ) -> Result<TrustedDeviceDto, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let key = self.ctx.require_key(&id)?;
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("give the device a name".into()));
        }
        let mut header = self.ctx.storage.read_header(&id)?;
        let device = header.trust_device(&key, name, self.ctx.now())?;
        self.save_header(&id, &header, &format!("vault: trust device {name}"))?;
        if for_this_device {
            self.ctx
                .kv
                .set(&keys::device_key(id.as_str()), &device.to_text())?;
        }
        self.ctx
            .record(&id, AuditKind::DeviceTrusted, None, Some(name));
        let entry = header.devices.last().map(|d| to_dto(d, for_this_device));
        Ok(TrustedDeviceDto {
            device: entry.ok_or_else(|| CoreError::Internal("device entry missing".into()))?,
            device_key: (!for_this_device).then(|| device.to_text()),
        })
    }

    /// The trusted devices of a vault (readable even when locked: the header is plain).
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn list_devices(&self, vault_id: &str) -> Result<Vec<DeviceDto>, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let header = self.ctx.storage.read_header(&id)?;
        let mine = self.local_device_id(&id);
        Ok(header
            .devices
            .iter()
            .map(|d| to_dto(d, mine.as_deref() == Some(d.id.as_str())))
            .collect())
    }

    /// Revokes a device: it can no longer unlock once it has synced the new header.
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] unless unlocked; [`CoreError::NotFound`] for an unknown id.
    pub fn revoke_device(&self, vault_id: &str, device_id: &str) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        self.ctx.require_key(&id)?;
        let mut header = self.ctx.storage.read_header(&id)?;
        if !header.revoke_device(device_id) {
            return Err(CoreError::NotFound { entity: "device" });
        }
        self.save_header(&id, &header, "vault: revoke device")?;
        if self.local_device_id(&id).as_deref() == Some(device_id) {
            self.ctx.kv.delete(&keys::device_key(id.as_str()))?;
        }
        self.ctx
            .record(&id, AuditKind::DeviceRevoked, None, Some(device_id));
        Ok(())
    }

    /// Unlocks with this device's key (or `device_key`, e.g. from `GITVAULT_DEVICE_KEY`).
    ///
    /// # Errors
    /// [`CoreError::VaultLocked`] when this device holds no key for the vault;
    /// [`CoreError::PermissionDenied`] when the key was revoked.
    pub fn unlock_trusted(
        &self,
        vault_id: &str,
        device_key: Option<&str>,
        context: &TrustContext<'_>,
    ) -> Result<VaultSummary, CoreError> {
        let id = VaultId::parse(vault_id)?;
        let text = match device_key {
            // An explicit key (servers, CI) is the operator's decision: no folder rule applies.
            Some(text) => text.to_owned(),
            None => {
                let text = self
                    .ctx
                    .kv
                    .get(&keys::device_key(id.as_str()))?
                    .ok_or(CoreError::VaultLocked)?;
                if !self.trust_policy(&id).allows(context) {
                    return Err(CoreError::VaultLocked);
                }
                text
            }
        };
        self.require_repo_access(&id)?;
        let header = self.ctx.storage.read_header(&id)?;
        let key = header.unlock_with_device(&DeviceKey::parse(&text)?)?;
        self.ctx.sessions.put(&id, key)?;
        self.set_current(&id)?;
        self.ctx
            .record(&id, AuditKind::Unlocked, None, Some("trusted device"));
        self.info(vault_id)
    }

    /// Trusts a folder (and its subfolders) on this device, optionally for some environments
    /// only. From then on this device unlocks by itself only inside trusted folders.
    ///
    /// # Errors
    /// [`CoreError::Validation`] when this device is not trusted yet.
    pub fn trust_folder(
        &self,
        vault_id: &str,
        folder: &Path,
        environments: &[String],
    ) -> Result<Vec<FolderRuleDto>, CoreError> {
        let id = VaultId::parse(vault_id)?;
        if self.local_device_id(&id).is_none() {
            return Err(CoreError::Validation(
                "this device is not trusted yet: run `gitvault device trust` first".into(),
            ));
        }
        let mut policy = self.trust_policy(&id);
        policy.add(FolderRule {
            path: folder.to_path_buf(),
            environments: environments.iter().map(|e| e.trim().to_owned()).collect(),
        });
        self.save_policy(&id, &policy)?;
        Ok(folder_dtos(&policy))
    }

    /// The folders this device is trusted in (empty and unrestricted means "anywhere").
    ///
    /// # Errors
    /// Fails on an invalid id.
    pub fn list_folders(&self, vault_id: &str) -> Result<Vec<FolderRuleDto>, CoreError> {
        let id = VaultId::parse(vault_id)?;
        Ok(folder_dtos(&self.trust_policy(&id)))
    }

    /// Stops trusting a folder.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] when the folder was not trusted.
    pub fn remove_folder(&self, vault_id: &str, folder: &Path) -> Result<(), CoreError> {
        let id = VaultId::parse(vault_id)?;
        let mut policy = self.trust_policy(&id);
        if !policy.remove(folder) {
            return Err(CoreError::NotFound { entity: "folder" });
        }
        self.save_policy(&id, &policy)
    }

    fn trust_policy(&self, id: &VaultId) -> TrustPolicy {
        self.ctx
            .kv
            .get(&keys::trust_policy(id.as_str()))
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    fn save_policy(&self, id: &VaultId, policy: &TrustPolicy) -> Result<(), CoreError> {
        let json = serde_json::to_string(policy).map_err(|e| CoreError::Internal(e.to_string()))?;
        self.ctx.kv.set(&keys::trust_policy(id.as_str()), &json)
    }

    fn local_device_id(&self, id: &VaultId) -> Option<String> {
        let text = self.ctx.kv.get(&keys::device_key(id.as_str())).ok()??;
        DeviceKey::parse(&text).ok().map(|k| k.id().to_owned())
    }

    fn save_header(
        &self,
        id: &VaultId,
        header: &VaultHeader,
        message: &str,
    ) -> Result<(), CoreError> {
        self.ctx.storage.write_header(id, header)?;
        self.ctx
            .vcs
            .commit_all(id, message, &self.ctx.author())
            .map(|_| ())
    }
}

fn folder_dtos(policy: &TrustPolicy) -> Vec<FolderRuleDto> {
    policy
        .folders
        .iter()
        .map(|rule| FolderRuleDto {
            path: rule.path.to_string_lossy().into_owned(),
            environments: rule.environments.clone(),
        })
        .collect()
}

fn to_dto(device: &crate::domain::vault::TrustedDevice, this_device: bool) -> DeviceDto {
    DeviceDto {
        id: device.id.clone(),
        name: device.name.clone(),
        created_at: device.created_at,
        this_device,
    }
}
