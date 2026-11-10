//! Self-description for humans and AI tools: version, command catalog, JSON Schemas, completions.

use anyhow::{anyhow, Result};
use clap::CommandFactory;
use gitvault_core::{
    domain::{audit::AuditEvent, settings::Settings},
    dto::{
        AccountDto, ConflictDto, CreatedVault, Facets, GeneratedDto, HistoryEntry,
        ImportPreviewEntry, ImportReport, RevealedSecret, SecretDetail, SecretSummary, SyncResult,
        SyncStatus, VaultList, VaultSummary,
    },
};
use schemars::schema_for;
use serde_json::{json, Value};

use crate::{app::App, cli::Cli};

pub fn version(app: &App) -> Result<()> {
    let info = json!({ "name": "gitvault", "version": env!("CARGO_PKG_VERSION") });
    app.emit(&info, || format!("gitvault {}", env!("CARGO_PKG_VERSION")))
}

/// Every command, its arguments and flags, as JSON.
pub fn commands(app: &App) -> Result<()> {
    let tree = describe(&Cli::command());
    app.emit(&tree, || {
        serde_json::to_string_pretty(&tree).unwrap_or_default()
    })
}

fn describe(command: &clap::Command) -> Value {
    let args: Vec<Value> = command
        .get_arguments()
        .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
        .map(|a| {
            json!({
                "name": a.get_id().as_str(),
                "long": a.get_long(),
                "short": a.get_short().map(String::from),
                "required": a.is_required_set(),
                "takes_value": a.get_action().takes_values(),
                "help": a.get_help().map(ToString::to_string),
            })
        })
        .collect();
    let subcommands: Vec<Value> = command
        .get_subcommands()
        .filter(|c| c.get_name() != "help")
        .map(describe)
        .collect();
    json!({
        "name": command.get_name(),
        "about": command.get_about().map(ToString::to_string),
        "args": args,
        "subcommands": subcommands,
    })
}

macro_rules! schema_registry {
    ($($ty:ident),* $(,)?) => {
        fn registry() -> Vec<(&'static str, Value)> {
            vec![$((stringify!($ty), serde_json::to_value(schema_for!($ty)).unwrap_or(Value::Null))),*]
        }
    };
}

schema_registry!(
    AccountDto,
    VaultSummary,
    VaultList,
    CreatedVault,
    SecretSummary,
    SecretDetail,
    RevealedSecret,
    HistoryEntry,
    Facets,
    ImportPreviewEntry,
    ImportReport,
    SyncStatus,
    SyncResult,
    ConflictDto,
    GeneratedDto,
    AuditEvent,
    Settings,
);

/// JSON Schema of an output type, or the list of available types.
pub fn schema(app: &App, name: Option<&str>) -> Result<()> {
    let all = registry();
    let Some(name) = name else {
        let names: Vec<&str> = all.iter().map(|(n, _)| *n).collect();
        return app.emit(&names, || names.join("\n"));
    };
    let (_, schema) = all
        .into_iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .ok_or_else(|| anyhow!("unknown type {name}: run `gitvault schema` to list them"))?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}

pub fn completions(shell: clap_complete::Shell) {
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        "gitvault",
        &mut std::io::stdout(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_nested_commands() {
        let tree = describe(&Cli::command());
        let text = tree.to_string();
        for expected in ["secret", "rotate", "run", "unlock", "mcp"] {
            assert!(
                text.contains(&format!("\"name\":\"{expected}\"")),
                "missing {expected}"
            );
        }
    }

    #[test]
    fn every_schema_serializes() {
        assert!(registry().iter().all(|(_, schema)| schema.is_object()));
    }
}
