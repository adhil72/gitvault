//! `trust`: limit where this device unlocks by itself. Bare `trust` asks which vault and
//! environments; `trust add | list | remove` is the scriptable form.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gitvault_core::{dto::FolderRuleDto, CoreError};

use crate::{
    app::App,
    cli::{PasswordSource, TrustArgs, TrustCmd},
    commands::device::unlock_vault,
    output::table,
    prompt,
};

const DEVICE_NAME: &str = "This computer";

pub fn run(app: &App, args: &TrustArgs) -> Result<()> {
    match &args.cmd {
        Some(TrustCmd::Add {
            path,
            environments,
            password,
        }) => add(app, path, environments, password),
        Some(TrustCmd::List) => list(app),
        Some(TrustCmd::Remove { path }) => remove(app, path),
        None => trust_here(app, args),
    }
}

fn resolve(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("{} is not a folder that exists", path.display()))
}

/// `gv trust`: asks the questions on a terminal, or trusts everything when scripted.
fn trust_here(app: &App, args: &TrustArgs) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    if app.interactive && args.environments.is_empty() {
        return guided(app, &path, &args.password);
    }
    add(app, &path, &args.environments, &args.password)
}

fn guided(app: &App, path: &Path, password: &PasswordSource) -> Result<()> {
    let folder = resolve(path)?;
    let vault_id = pick_vault(app)?;
    unlock_vault(app, &vault_id, password, true)?;
    let environments = pick_environments(app, &vault_id)?;
    let scope = if environments.is_empty() {
        "all environments".to_owned()
    } else {
        environments.join(", ")
    };
    if !prompt::confirm(&format!("\nTrust {} for {scope}?", folder.display()))? {
        return app.emit(&serde_json::json!({ "trusted": false }), || {
            "Nothing changed.".into()
        });
    }
    apply(app, &vault_id, &folder, &environments, password)
}

fn pick_vault(app: &App) -> Result<String> {
    if app.vault_flag().is_some() {
        return app.vault_id();
    }
    let vaults: Vec<_> = app
        .services
        .vaults
        .list()?
        .vaults
        .into_iter()
        .filter(|v| v.id.is_some())
        .collect();
    let labels: Vec<String> = vaults
        .iter()
        .map(|v| {
            v.repo
                .as_ref()
                .map_or_else(|| v.name.clone(), |r| format!("{} ({r})", v.name))
        })
        .collect();
    let picked =
        match vaults.len() {
            0 => return Err(CoreError::Validation(
                "no vault on this device: run `gv vault create` or `gv vault connect owner/name`"
                    .into(),
            )
            .into()),
            1 => 0,
            _ => {
                let current = vaults.iter().position(|v| v.current).unwrap_or(0);
                prompt::choose_one("Which vault?", &labels, current)?
            }
        };
    vaults[picked]
        .id
        .clone()
        .context("the vault has no id on this device")
}

/// Empty means every environment (also when the user picks them all, so new ones are covered).
fn pick_environments(app: &App, vault_id: &str) -> Result<Vec<String>> {
    let names: Vec<String> = app
        .services
        .secrets
        .facets(vault_id)?
        .environments
        .into_iter()
        .map(|e| e.name)
        .collect();
    if names.is_empty() {
        app.note("This vault has no environments yet, so the folder is trusted for all of them.");
        return Ok(Vec::new());
    }
    let picked = prompt::choose_many("Which environments?", &names)?;
    if picked.len() == names.len() {
        return Ok(Vec::new());
    }
    Ok(picked.into_iter().map(|i| names[i].clone()).collect())
}

fn add(app: &App, path: &Path, environments: &[String], password: &PasswordSource) -> Result<()> {
    let folder = resolve(path)?;
    apply(app, &app.vault_id()?, &folder, environments, password)
}

fn apply(
    app: &App,
    vault_id: &str,
    folder: &Path,
    environments: &[String],
    password: &PasswordSource,
) -> Result<()> {
    let devices = app.services.vaults.list_devices(vault_id)?;
    if !devices.iter().any(|d| d.this_device) {
        // Needs the master password once, like `device trust`.
        unlock_vault(app, vault_id, password, false)?;
        app.services
            .vaults
            .trust_device(vault_id, DEVICE_NAME, true)?;
    }
    let rules = app
        .services
        .vaults
        .trust_folder(vault_id, folder, environments)?;
    app.emit(&rules, || {
        format!(
            "Trusted {} (and its subfolders): `gv run` and `gv env export` unlock here without a password.",
            folder.display()
        )
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
