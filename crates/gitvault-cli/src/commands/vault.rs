use anyhow::Result;
use gitvault_core::dto::{CreateVaultInput, RemoteStatus, UnlockInput, VaultState, VaultSummary};

use crate::{
    app::App,
    cli::{CreateVaultArgs, PasswordArgs, UnlockArgs, VaultCmd},
    input,
    output::table,
};

pub fn run(app: &App, cmd: &VaultCmd) -> Result<()> {
    match cmd {
        VaultCmd::List { remote } => list(app, *remote),
        VaultCmd::Create(args) => create(app, args),
        VaultCmd::Use { id } => show(app, &app.services.vaults.use_vault(id)?),
        VaultCmd::Info => show(app, &app.services.vaults.info(&app.vault_id()?)?),
        VaultCmd::Connect { repo } => show(app, &app.services.vaults.connect(repo)?),
        VaultCmd::Forget { id } => {
            app.services.vaults.forget(id)?;
            app.emit(&serde_json::json!({ "forgotten": id }), || {
                format!("Removed the local copy of {id}.")
            })
        }
        VaultCmd::Remote => remote(app),
        VaultCmd::Republish => show(app, &app.services.vaults.republish(&app.vault_id()?)?),
        VaultCmd::Password(args) => change_password(app, args),
        VaultCmd::Recovery(args) => new_recovery_key(app, args),
    }
}

fn remote(app: &App) -> Result<()> {
    let status = app.services.vaults.remote_status(&app.vault_id()?)?;
    app.emit(&status, || match status {
        RemoteStatus::LocalOnly => "No GitHub repository (local vault).".to_owned(),
        RemoteStatus::Reachable => "The GitHub repository is reachable.".to_owned(),
        RemoteStatus::Missing => {
            "The GitHub repository was deleted. Run `gitvault vault republish` to recreate it, or `gitvault vault forget <id>` to remove this copy.".to_owned()
        }
        RemoteStatus::NoAccess => "Your GitHub account cannot see the repository.".to_owned(),
        RemoteStatus::Offline => "Could not reach GitHub.".to_owned(),
    })
}

fn list(app: &App, remote: bool) -> Result<()> {
    let mut list = app.services.vaults.list()?;
    if remote {
        let found = app.services.vaults.discover()?;
        list.vaults.extend(found.vaults);
        list.remote_error = found.remote_error;
    }
    if let Some(error) = &list.remote_error {
        app.note(&format!("warning: could not reach GitHub: {error}"));
    }
    app.emit(&list, || {
        let rows: Vec<Vec<String>> = list.vaults.iter().map(row).collect();
        table(&["", "ID", "NAME", "REPOSITORY", "STATE"], &rows)
    })
}

fn row(vault: &VaultSummary) -> Vec<String> {
    let state = match (&vault.state, vault.unlocked) {
        (VaultState::RemoteOnly, _) => "on GitHub only (gitvault vault connect)",
        (VaultState::Local, true) => "unlocked",
        (VaultState::Local, false) => "locked",
    };
    vec![
        if vault.current { "*" } else { "" }.into(),
        vault.id.clone().unwrap_or_else(|| "-".into()),
        vault.name.clone(),
        vault.repo.clone().unwrap_or_else(|| "-".into()),
        state.into(),
    ]
}

fn show(app: &App, vault: &VaultSummary) -> Result<()> {
    app.emit(vault, || {
        table(&["", "ID", "NAME", "REPOSITORY", "STATE"], &[row(vault)])
    })
}

fn create(app: &App, args: &CreateVaultArgs) -> Result<()> {
    let password = input::new_password(app, args.password.password_stdin)?;
    let signed_in = app.services.auth.status().is_some();
    if !args.local_only && !signed_in {
        app.note(
            "Not signed in to GitHub: creating a local vault only (run `gitvault auth login`).",
        );
    }
    let created = app.services.vaults.create(&CreateVaultInput {
        name: args.name.clone(),
        description: args.description.clone(),
        repo_name: args.repo_name.clone(),
        create_remote: !args.local_only && signed_in,
        master_password: password,
    })?;
    if let Some(error) = &created.remote_error {
        app.note(&format!(
            "warning: the vault was created locally but pushing to GitHub failed: {error}"
        ));
    }
    app.emit(&created, || {
        format!(
            "Created vault {} ({}).\n\nRecovery key (store it offline; it is shown only once):\n  {}",
            created.vault.name,
            created.vault.id.clone().unwrap_or_default(),
            created.recovery_key
        )
    })
}

pub fn unlock(app: &App, args: &UnlockArgs) -> Result<()> {
    let vault_id = app.vault_id()?;
    let (password, recovery_key) = if args.recovery_key_stdin {
        (None, Some(input::stdin_line()?))
    } else {
        (
            Some(input::password(
                app,
                "Master password: ",
                args.password.password_stdin,
            )?),
            None,
        )
    };
    let summary = app.services.vaults.unlock(&UnlockInput {
        vault_id: vault_id.clone(),
        password,
        recovery_key,
    })?;
    let token = if args.export {
        Some(app.services.vaults.export_session(&vault_id)?)
    } else {
        None
    };
    app.emit(
        &serde_json::json!({ "vault": summary, "session": token }),
        || match &token {
            Some(t) => format!("GITVAULT_SESSION={t}"),
            None => format!("Unlocked {}.", summary.name),
        },
    )
}

pub fn lock(app: &App) -> Result<()> {
    let vault_id = app.vault_id()?;
    app.services.vaults.lock(&vault_id)?;
    app.emit(&serde_json::json!({ "locked": vault_id }), || {
        "Locked.".into()
    })
}

fn change_password(app: &App, args: &PasswordArgs) -> Result<()> {
    let vault_id = app.vault_id()?;
    let current = input::password(
        app,
        "Current master password: ",
        args.password.password_stdin,
    )?;
    let new = input::new_password(app, false)
        .or_else(|_| input::password(app, "New master password: ", true))?;
    app.services
        .vaults
        .change_password(&vault_id, &current, &new)?;
    app.emit(&serde_json::json!({ "changed": true }), || {
        "Master password changed.".into()
    })
}

fn new_recovery_key(app: &App, args: &PasswordArgs) -> Result<()> {
    let vault_id = app.vault_id()?;
    let current = input::password(app, "Master password: ", args.password.password_stdin)?;
    let key = app
        .services
        .vaults
        .regenerate_recovery_key(&vault_id, &current)?;
    app.emit(&serde_json::json!({ "recovery_key": key }), || {
        format!("New recovery key (the old one no longer works):\n  {key}")
    })
}
