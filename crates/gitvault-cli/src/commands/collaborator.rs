//! `collaborator list | invite | remove`: GitHub users with access to the vault's repository.

use anyhow::Result;
use gitvault_core::{
    domain::collaborator::CollaboratorRole,
    dto::{CollaboratorDto, InviteCollaboratorInput},
};

use crate::{app::App, cli::CollaboratorCmd, output::table};

pub fn run(app: &App, cmd: &CollaboratorCmd) -> Result<()> {
    let vault_id = app.vault_id()?;
    match cmd {
        CollaboratorCmd::List => list(app, &vault_id),
        CollaboratorCmd::Invite { login, write } => invite(app, vault_id, login, *write),
        CollaboratorCmd::Remove { login } => remove(app, &vault_id, login),
    }
}

fn list(app: &App, vault_id: &str) -> Result<()> {
    let people = app.services.vaults.list_collaborators(vault_id)?;
    app.emit(&people, || {
        if people.is_empty() {
            return "No collaborators yet. Invite one with `gitvault collaborator invite <login>`."
                .into();
        }
        let rows: Vec<Vec<String>> = people.iter().map(row).collect();
        table(&["LOGIN", "ROLE", "STATUS"], &rows)
    })
}

fn row(person: &CollaboratorDto) -> Vec<String> {
    vec![
        person.login.clone(),
        role_name(person.role).into(),
        if person.pending { "invited" } else { "active" }.into(),
    ]
}

fn invite(app: &App, vault_id: String, login: &str, write: bool) -> Result<()> {
    let role = if write {
        CollaboratorRole::Write
    } else {
        CollaboratorRole::Read
    };
    app.services
        .vaults
        .invite_collaborator(&InviteCollaboratorInput {
            vault_id,
            login: login.into(),
            role,
        })?;
    app.emit(&serde_json::json!({ "invited": login, "role": role_name(role) }), || {
        format!(
            "Invited {login} as {}. They still need the master password (or a trusted-device key) to open the vault.",
            role_name(role)
        )
    })
}

fn remove(app: &App, vault_id: &str, login: &str) -> Result<()> {
    app.services.vaults.remove_collaborator(vault_id, login)?;
    app.emit(&serde_json::json!({ "removed": login }), || {
        format!("{login} no longer has access to the vault's repository.")
    })
}

const fn role_name(role: CollaboratorRole) -> &'static str {
    match role {
        CollaboratorRole::Read => "read",
        CollaboratorRole::Write => "write",
    }
}
