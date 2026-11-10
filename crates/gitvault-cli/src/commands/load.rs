//! `env export`: the variables of one environment as JSON, for app loaders.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

use crate::app::App;

#[derive(Serialize)]
struct Loaded<'a> {
    environment: &'a str,
    variables: BTreeMap<String, String>,
}

/// Prints `{ "environment": ..., "variables": { NAME: value } }` (always JSON, values included:
/// this command exists to hand them to a trusted loader, so it never logs anything else).
pub fn export(app: &App, environment: &str) -> Result<()> {
    let vault_id = app.ensure_unlocked(Some(environment))?;
    let variables = app
        .services
        .secrets
        .resolve_environment(&vault_id, environment)?
        .into_iter()
        .map(|entry| (entry.name, entry.value))
        .collect();
    println!(
        "{}",
        serde_json::to_string(&Loaded {
            environment,
            variables
        })?
    );
    Ok(())
}
