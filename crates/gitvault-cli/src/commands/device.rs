//! `device trust | list | revoke`: unlock without the master password on trusted devices.

use anyhow::Result;
use gitvault_core::{
    dto::{DeviceDto, UnlockInput},
    CoreError,
};

use crate::{
    app::App,
    cli::{DeviceCmd, PasswordSource},
    input,
    output::table,
};

pub fn run(app: &App, cmd: &DeviceCmd) -> Result<()> {
    match cmd {
        DeviceCmd::Trust {
            name,
            export,
            password,
        } => trust(app, name, *export, password),
        DeviceCmd::List => list(app),
        DeviceCmd::Revoke { id, password } => revoke(app, id, password),
    }
}

/// Makes sure the vault is unlocked. Revoking may use a trusted device; trusting a new device
/// needs the master password, so a trusted device cannot mint more trust by itself.
pub fn unlocked_vault(app: &App, password: &PasswordSource, allow_trusted: bool) -> Result<String> {
    let vault_id = app.vault_id()?;
    if !app.services.vaults.info(&vault_id)?.unlocked {
        if allow_trusted {
            match app.unlock_trusted_here(&vault_id, None, None) {
                Ok(_) => return Ok(vault_id),
                Err(CoreError::VaultLocked) => {}
                Err(other) => return Err(other.into()),
            }
        }
        let password = input::password(app, "Master password: ", password.password_stdin)?;
        app.services.vaults.unlock(&UnlockInput {
            vault_id: vault_id.clone(),
            password: Some(password),
            recovery_key: None,
        })?;
    }
    Ok(vault_id)
}

fn trust(app: &App, name: &str, export: bool, password: &PasswordSource) -> Result<()> {
    let vault_id = unlocked_vault(app, password, false)?;
    let result = app.services.vaults.trust_device(&vault_id, name, !export)?;
    app.emit(&result, || match &result.device_key {
        Some(key) => format!(
            "Device key for {name} (shown once; set it as GITVAULT_DEVICE_KEY on that machine):\n{key}"
        ),
        None => format!("This device ({name}) is now trusted: it unlocks without a password."),
    })
}

fn list(app: &App) -> Result<()> {
    let devices = app.services.vaults.list_devices(&app.vault_id()?)?;
    app.emit(&devices, || {
        let rows: Vec<Vec<String>> = devices.iter().map(row).collect();
        table(&["", "ID", "NAME"], &rows)
    })
}

fn row(device: &DeviceDto) -> Vec<String> {
    vec![
        if device.this_device { "*" } else { "" }.into(),
        device.id.clone(),
        device.name.clone(),
    ]
}

fn revoke(app: &App, id: &str, password: &PasswordSource) -> Result<()> {
    let vault_id = unlocked_vault(app, password, true)?;
    app.services.vaults.revoke_device(&vault_id, id)?;
    app.emit(&serde_json::json!({ "revoked": id }), || {
        format!("Device {id} is no longer trusted.")
    })
}
