use gitvault_core::dto::{
    CreateVaultInput, CreatedVault, DeviceDto, TrustedDeviceDto, UnlockInput, VaultList,
    VaultSummary,
};

use super::command;

command!(vault_list() -> VaultList => |s| s.vaults.list());
command!(vault_discover() -> VaultList => |s| s.vaults.discover());
command!(vault_create(input: CreateVaultInput) -> CreatedVault => |s| s.vaults.create(&input));
command!(vault_connect(repo: String) -> VaultSummary => |s| s.vaults.connect(&repo));
command!(vault_unlock(input: UnlockInput) -> VaultSummary => |s| s.vaults.unlock(&input));
command!(vault_unlock_trusted(vault_id: String) -> VaultSummary => |s| s.vaults.unlock_trusted(&vault_id, None, &Default::default()));
command!(device_trust(vault_id: String, name: String) -> TrustedDeviceDto => |s| s.vaults.trust_device(&vault_id, &name, true));
command!(device_list(vault_id: String) -> Vec<DeviceDto> => |s| s.vaults.list_devices(&vault_id));
command!(device_revoke(vault_id: String, device_id: String) -> () => |s| s.vaults.revoke_device(&vault_id, &device_id));
command!(vault_lock(vault_id: String) -> () => |s| s.vaults.lock(&vault_id));
command!(vault_info(vault_id: String) -> VaultSummary => |s| s.vaults.info(&vault_id));
command!(vault_use(vault_id: String) -> VaultSummary => |s| s.vaults.use_vault(&vault_id));
command!(vault_forget(vault_id: String) -> () => |s| s.vaults.forget(&vault_id));
command!(vault_current() -> Option<String> => |s| Ok(s.vaults.current()));
command!(vault_change_password(vault_id: String, current: String, new_password: String) -> () => |s| {
    s.vaults.change_password(&vault_id, &current, &new_password)
});
command!(vault_regenerate_recovery(vault_id: String, current: String) -> String => |s| {
    s.vaults.regenerate_recovery_key(&vault_id, &current)
});
