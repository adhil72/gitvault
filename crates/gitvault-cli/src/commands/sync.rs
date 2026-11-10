use anyhow::Result;
use gitvault_core::dto::{ConflictKeep, SyncResult, SyncStatus};

use crate::{app::App, cli::SyncCmd, output::table, time};

pub fn run(app: &App, cmd: &SyncCmd) -> Result<()> {
    let vault = app.vault_id()?;
    match cmd {
        SyncCmd::Now => sync_now(app, &vault),
        SyncCmd::Status => status(app, &vault),
        SyncCmd::Conflicts => conflicts(app, &vault),
        SyncCmd::Resolve { secret, keep } => resolve(app, &vault, secret, keep),
    }
}

fn describe(result: &SyncResult) -> String {
    if result.no_remote {
        "This vault has no GitHub repository.".into()
    } else if result.conflicts > 0 {
        format!(
            "{} conflict(s): see `gitvault sync conflicts`.",
            result.conflicts
        )
    } else {
        format!(
            "Synced: {} pulled, {} pushed.",
            result.pulled, result.pushed
        )
    }
}

fn sync_now(app: &App, vault: &str) -> Result<()> {
    let result = app.services.sync.sync_now(vault)?;
    app.emit(&result, || describe(&result))
}

fn describe_status(status: &SyncStatus) -> String {
    format!(
        "{:?}: {} pending, {} conflicts, last sync {}",
        status.state,
        status.pending,
        status.conflicts,
        status.last_sync.map_or("never".into(), time::format_time)
    )
}

fn status(app: &App, vault: &str) -> Result<()> {
    let status = app.services.sync.status(vault)?;
    app.emit(&status, || describe_status(&status))
}

fn conflicts(app: &App, vault: &str) -> Result<()> {
    let conflicts = app.services.sync.conflicts(vault)?;
    app.emit(&conflicts, || {
        let rows: Vec<Vec<String>> = conflicts
            .iter()
            .map(|c| {
                vec![
                    c.secret_id.clone(),
                    c.name.clone().unwrap_or_else(|| "-".into()),
                ]
            })
            .collect();
        table(&["SECRET ID", "NAME"], &rows)
    })
}

fn resolve(app: &App, vault: &str, secret: &str, keep: &str) -> Result<()> {
    let side = if keep == "local" {
        ConflictKeep::Local
    } else {
        ConflictKeep::Remote
    };
    app.services.sync.resolve(vault, secret, &side)?;
    app.emit(&serde_json::json!({ "resolved": secret }), || {
        format!("Kept the {keep} version. Run `gitvault sync now`.")
    })
}
