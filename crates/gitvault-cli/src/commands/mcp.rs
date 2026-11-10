//! A minimal Model Context Protocol server over stdio (newline-delimited JSON-RPC).
//!
//! Safe by default: tools expose metadata only. Plaintext values are never
//! returned; `run_command` injects secrets into a child process and redacts
//! every secret value from its output.

use std::{
    io::{BufRead, Write},
    process::Command,
};

use anyhow::Result;
use gitvault_core::dto::{EnvFilter, ListSecretsQuery};
use serde_json::{json, Value};

use crate::{app::App, commands::run::redact};

const MAX_OUTPUT_BYTES: usize = 100_000;

pub fn serve(app: &App) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = handle_line(app, &line) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn handle_line(app: &App, line: &str) -> Option<Value> {
    let request: Value = serde_json::from_str(line).ok()?;
    let id = request.get("id").cloned()?; // notifications have no id and get no answer
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let body = match method {
        "initialize" => Ok(initialize()),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => Ok(call(app, &params)),
        "ping" => Ok(json!({})),
        other => Err(json!({ "code": -32601, "message": format!("method not found: {other}") })),
    };
    Some(match body {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

fn initialize() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "gitvault", "version": env!("CARGO_PKG_VERSION") },
    })
}

fn tool(name: &str, description: &str, properties: &Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": { "type": "object", "properties": properties } })
}

fn tools() -> Vec<Value> {
    let filter = json!({ "environment": {"type":"string"}, "tag": {"type":"string"} });
    vec![
        tool("list_secrets", "List secret metadata (names, environments). Values are never returned.", &filter),
        tool("search_secrets", "Search secret names, environments and tags.", &json!({ "text": {"type":"string"} })),
        tool("sync_status", "Show whether the vault is synced with GitHub.", &json!({})),
        tool(
            "run_command",
            "Run a command with secrets injected as environment variables. Secret values are redacted from the output.",
            &json!({ "command": {"type":"array","items":{"type":"string"}}, "environment": {"type":"string"}, "tag": {"type":"string"} }),
        ),
    ]
}

fn call(app: &App, params: &Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);
    match dispatch(app, name, &args) {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }] }),
        Err(error) => {
            json!({ "isError": true, "content": [{ "type": "text", "text": error.to_string() }] })
        }
    }
}

fn arg(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn dispatch(app: &App, name: &str, args: &Value) -> Result<String> {
    let vault = app.vault_id()?;
    match name {
        "list_secrets" | "search_secrets" => {
            let items = app.services.secrets.list(&ListSecretsQuery {
                vault_id: vault,
                search: arg(args, "text"),
                environment: arg(args, "environment"),
                tag: arg(args, "tag"),
                kind: None,
                favorites_only: false,
                expiring_within_days: None,
                sort: None,
            })?;
            Ok(serde_json::to_string_pretty(&items)?)
        }
        "sync_status" => Ok(serde_json::to_string_pretty(
            &app.services.sync.status(&vault)?,
        )?),
        "run_command" => run_command(app, &vault, args),
        other => anyhow::bail!("unknown tool: {other}"),
    }
}

fn run_command(app: &App, vault: &str, args: &Value) -> Result<String> {
    let command: Vec<String> = args
        .get("command")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let (program, rest) = command
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("command must not be empty"))?;
    let filter = EnvFilter {
        environment: arg(args, "environment"),
        tag: arg(args, "tag"),
    };
    let entries = app.services.secrets.resolve_env(vault, &filter)?;
    let output = Command::new(program)
        .args(rest)
        .envs(entries.iter().map(|e| (&e.name, &e.value)))
        .output()?;
    let mut text = format!(
        "exit code: {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    text.truncate(MAX_OUTPUT_BYTES);
    Ok(redact(&text, &entries))
}
