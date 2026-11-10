//! `trust add | list | remove`: limit where this device unlocks by itself.

use std::path::Path;

use anyhow::{Context, Result};
use gitvault_core::dto::FolderRuleDto;

use crate::{
    app::App,
    cli::{PasswordSource, TrustCmd},
    commands::device::unlocked_vault,
    output::table,
};

const DEVICE_NAME: &str = "This computer";

pub fn run(app: &App, cmd: &TrustCmd) -> Result<()> {
    match cmd {
        TrustCmd::Add {
            path,
            environments,
            password,
        } => add(app, path, environments, password),
        TrustCmd::List => list(app),
        TrustCmd::Remove { path } => remove(app, path),
    }
}

fn resolve(path: &Path) -> Result<std::path::PathBuf> {
    path.canonicalize()
        .with_context(|| format!("{} is not a folder that exists", path.display()))
}

fn add(app: &App, path: &Path, environments: &[String], password: &PasswordSource) -> Result<()> {
    let folder = resolve(path)?;
    let vault_id = app.vault_id()?;
    let devices = app.services.vaults.list_devices(&vault_id)?;
    if !devices.iter().any(|d| d.this_device) {
        // Needs the master password once, like `device trust`.
        let vault_id = unlocked_vault(app, password, false)?;
        app.services
            .vaults
            .trust_device(&vault_id, DEVICE_NAME, true)?;
    }
    let rules = app
        .services
        .vaults
        .trust_folder(&vault_id, &folder, environments)?;
    app.emit(&rules, || {
        format!("Trusted {} (and its subfolders).", folder.display())
    })
}

fn list(app: &App) -> Result<()> {
    let rules = app.services.vaults.list_folders(&app.vault_id()?)?;
    app.emit(&rules, || {
        let rows: Vec<Vec<String>> = rules.iter().map(row).collect();
        table(&["FOLDER", "ENVIRONMENTS"], &rows)
    })
}

fn row(rule: &FolderRuleDto) -> Vec<String> {
    let environments = if rule.environments.is_empty() {
        "all".to_owned()
    } else {
        rule.environments.join(", ")
    };
    vec![rule.path.clone(), environments]
}

fn remove(app: &App, path: &Path) -> Result<()> {
    let folder = resolve(path)?;
    app.services
        .vaults
        .remove_folder(&app.vault_id()?, &folder)?;
    app.emit(&serde_json::json!({ "removed": folder }), || {
        format!("No longer trusting {}.", folder.display())
    })
}
