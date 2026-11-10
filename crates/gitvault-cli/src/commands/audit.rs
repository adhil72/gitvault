use anyhow::{anyhow, Result};
use gitvault_core::{domain::audit::AuditKind, dto::AuditQuery};

use crate::{app::App, cli::AuditCmd, output::table, time};

fn parse_kind(text: &str) -> Result<AuditKind> {
    serde_json::from_value(serde_json::Value::String(text.to_lowercase()))
        .map_err(|_| anyhow!("unknown event kind: {text}"))
}

pub fn run(app: &App, cmd: &AuditCmd) -> Result<()> {
    let AuditCmd::List {
        kind,
        search,
        limit,
        all_vaults,
    } = cmd;
    let events = app.services.audit.list(&AuditQuery {
        vault_id: if *all_vaults {
            None
        } else {
            Some(app.vault_id()?)
        },
        kind: kind.as_deref().map(parse_kind).transpose()?,
        search: search.clone(),
        limit: Some(*limit),
    })?;
    app.emit(&events, || {
        let rows: Vec<Vec<String>> = events
            .iter()
            .map(|e| {
                vec![
                    time::format_time(e.at),
                    format!("{:?}", e.kind),
                    e.secret_name.clone().unwrap_or_else(|| "-".into()),
                    e.device.clone(),
                ]
            })
            .collect();
        table(&["TIME (UTC)", "EVENT", "SECRET", "DEVICE"], &rows)
    })
}
