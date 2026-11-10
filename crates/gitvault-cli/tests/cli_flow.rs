//! Runs the real `gitvault` binary against a temporary home directory.
#![allow(clippy::unwrap_used, clippy::panic, clippy::too_many_lines)]

use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;

const PASSWORD: &str = "correct horse battery";

fn run(home: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitvault"))
        .args(args)
        .env("GITVAULT_HOME", home)
        .env_remove("GITVAULT_SESSION")
        .env_remove("GITVAULT_VAULT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("not JSON: {}", String::from_utf8_lossy(&output.stdout)))
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Creates a local vault and returns its id (it also becomes the default vault).
fn create_vault(home: &Path) -> String {
    let out = run(
        home,
        &[
            "vault",
            "create",
            "--name",
            "Demo",
            "--local-only",
            "--password-stdin",
            "--json",
        ],
        &format!("{PASSWORD}\n"),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let created = json(&out);
    assert!(created["recovery_key"].as_str().unwrap().split('-').count() == 8);
    created["vault"]["id"].as_str().unwrap().to_owned()
}

#[test]
fn secret_workflow_and_safe_defaults() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());

    let add = run(
        home.path(),
        &[
            "secret",
            "add",
            "STRIPE_KEY",
            "--environment",
            "production",
            "--value-stdin",
        ],
        "sk_live_abcdef123456",
    );
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );

    let masked = run(home.path(), &["secret", "get", "STRIPE_KEY", "--json"], "");
    assert!(masked.status.success());
    assert!(
        !text(&masked).contains("sk_live_abcdef123456"),
        "value must be masked by default"
    );
    assert!(json(&masked)["value"].is_null());

    let raw = run(home.path(), &["secret", "get", "STRIPE_KEY", "--raw"], "");
    assert_eq!(text(&raw).trim(), "sk_live_abcdef123456");

    let ran = run(
        home.path(),
        &[
            "run",
            "--environment",
            "production",
            "--redact",
            "--",
            "sh",
            "-c",
            "echo key=$STRIPE_KEY",
        ],
        "",
    );
    assert_eq!(text(&ran).trim(), "key=[REDACTED]");
}

#[test]
fn locked_vault_gives_exit_code_four_and_a_json_error() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    assert!(run(home.path(), &["lock"], "").status.success());

    let out = run(home.path(), &["secret", "list", "--json"], "");
    assert_eq!(out.status.code(), Some(4));
    let error = json(&out);
    assert_eq!(error["error"]["code"], "E_LOCKED");
    assert_eq!(error["error"]["hint"], "gitvault unlock");

    let unlocked = run(
        home.path(),
        &["unlock", "--password-stdin"],
        &format!("{PASSWORD}\n"),
    );
    assert!(unlocked.status.success());
    assert!(run(home.path(), &["secret", "list", "--json"], "")
        .status
        .success());
}

#[test]
fn wrong_password_has_its_own_exit_code() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    run(home.path(), &["lock"], "");
    let out = run(
        home.path(),
        &["unlock", "--password-stdin"],
        "not the password\n",
    );
    assert_eq!(out.status.code(), Some(10));
}

#[test]
fn ai_tools_can_discover_commands_and_schemas() {
    let home = tempfile::tempdir().unwrap();
    let commands = json(&run(home.path(), &["commands", "--json"], ""));
    let names = commands.to_string();
    for expected in ["secret", "rotate", "run", "unlock", "mcp", "schema"] {
        assert!(
            names.contains(&format!("\"name\":\"{expected}\"")),
            "missing {expected}"
        );
    }
    let list = run(home.path(), &["schema", "--json"], "");
    assert!(text(&list).contains("SecretSummary"));
    let schema = json(&run(home.path(), &["schema", "SecretSummary"], ""));
    assert!(schema["properties"]["name"].is_object());
}

#[test]
fn generator_works_without_a_vault() {
    let home = tempfile::tempdir().unwrap();
    let out = run(
        home.path(),
        &["generate", "password", "--length", "24", "--json"],
        "",
    );
    assert!(out.status.success());
    assert_eq!(json(&out)["value"].as_str().unwrap().chars().count(), 24);
}

#[test]
fn mcp_server_lists_tools_and_never_exposes_values() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    run(
        home.path(),
        &["secret", "add", "TOKEN_X", "--value-stdin"],
        "super-secret-value-123",
    );
    let requests = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_secrets","arguments":{}}}"#,
        "\n",
    );
    let out = run(home.path(), &["mcp"], requests);
    let lines: Vec<Value> = text(&out)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3, "notifications get no response");
    assert!(lines[1]["result"]["tools"].as_array().unwrap().len() >= 4);
    let listing = lines[2]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(listing.contains("TOKEN_X"));
    assert!(!text(&out).contains("super-secret-value-123"));
}

#[test]
fn env_export_unlocks_from_the_environment_and_merges_shared_secrets() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    for (name, environment, value) in [
        ("SHARED", None, "shared"),
        ("KEY", Some("production"), "prod"),
        ("KEY", Some("development"), "dev"),
    ] {
        let mut args = vec!["secret", "add", name, "--value-stdin"];
        if let Some(environment) = environment {
            args.extend(["--environment", environment]);
        }
        assert!(run(home.path(), &args, value).status.success());
    }
    run(home.path(), &["lock"], "");

    let locked = run(home.path(), &["--json", "env", "export", "production"], "");
    assert_eq!(
        locked.status.code(),
        Some(4),
        "no password means E_LOCKED, never a prompt"
    );

    let out = Command::new(env!("CARGO_BIN_EXE_gitvault"))
        .args(["--json", "env", "export", "production"])
        .env("GITVAULT_HOME", home.path())
        .env("GITVAULT_PASSWORD", PASSWORD)
        .env_remove("GITVAULT_SESSION")
        .output()
        .unwrap();
    let body = json(&out);
    assert_eq!(body["variables"]["KEY"], "prod");
    assert_eq!(body["variables"]["SHARED"], "shared");
    assert_eq!(body["variables"].as_object().unwrap().len(), 2);
}

fn export_env(home: &Path, device_key: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gitvault"));
    command
        .args(["--json", "env", "export", "production"])
        .env("GITVAULT_HOME", home)
        .env_remove("GITVAULT_SESSION")
        .env_remove("GITVAULT_PASSWORD")
        .env_remove("GITVAULT_DEVICE_KEY");
    if let Some(key) = device_key {
        command.env("GITVAULT_DEVICE_KEY", key);
    }
    command.output().unwrap()
}

#[test]
fn trusted_devices_unlock_without_a_password_until_revoked() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    run(
        home.path(),
        &[
            "secret",
            "add",
            "KEY",
            "--environment",
            "production",
            "--value-stdin",
        ],
        "prod",
    );

    // Trust this device, then lock: it unlocks by itself.
    let trust = run(
        home.path(),
        &["--json", "device", "trust", "--name", "laptop"],
        "",
    );
    assert!(
        trust.status.success(),
        "{}",
        String::from_utf8_lossy(&trust.stderr)
    );
    run(home.path(), &["lock"], "");
    assert_eq!(
        json(&export_env(home.path(), None))["variables"]["KEY"],
        "prod"
    );

    // A key for a server works through the environment variable.
    run(home.path(), &["lock"], "");
    let exported = json(&run(
        home.path(),
        &[
            "--json",
            "device",
            "trust",
            "--name",
            "server",
            "--export",
            "--password-stdin",
        ],
        &format!("{PASSWORD}\n"),
    ));
    let server_key = exported["device_key"].as_str().unwrap().to_owned();

    // Revoke this device: locked again (exit 4).
    let devices = json(&run(home.path(), &["--json", "device", "list"], ""));
    let mine = devices
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["this_device"] == true)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(run(home.path(), &["device", "revoke", &mine], "")
        .status
        .success());
    run(home.path(), &["lock"], "");
    assert_eq!(export_env(home.path(), None).status.code(), Some(4));

    // The server key still works, until it is revoked too (exit 8).
    assert_eq!(
        json(&export_env(home.path(), Some(&server_key)))["variables"]["KEY"],
        "prod"
    );
    let server_id = server_key.split('.').nth(1).unwrap().to_owned();
    assert!(run(home.path(), &["device", "revoke", &server_id], "")
        .status
        .success());
    run(home.path(), &["lock"], "");
    assert_eq!(
        export_env(home.path(), Some(&server_key)).status.code(),
        Some(8)
    );
}

fn export_in(home: &Path, environment: &str, cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gitvault"))
        .args(["--json", "env", "export", environment])
        .current_dir(cwd)
        .env("GITVAULT_HOME", home)
        .env_remove("GITVAULT_SESSION")
        .env_remove("GITVAULT_PASSWORD")
        .env_remove("GITVAULT_DEVICE_KEY")
        .output()
        .unwrap()
}

#[test]
fn folder_trust_limits_silent_unlock_to_a_folder_and_environment() {
    let home = tempfile::tempdir().unwrap();
    create_vault(home.path());
    for (environment, value) in [("development", "dev"), ("production", "prod")] {
        run(
            home.path(),
            &[
                "secret",
                "add",
                "KEY",
                "--environment",
                environment,
                "--value-stdin",
            ],
            value,
        );
    }
    let work = tempfile::tempdir().unwrap();
    let project = work.path().join("shop");
    let nested = project.join("api/src");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir_all(work.path().join("shop-other")).unwrap();

    let trusted = run(
        home.path(),
        &[
            "trust",
            "add",
            project.to_str().unwrap(),
            "--env",
            "development",
            "--password-stdin",
        ],
        &format!("{PASSWORD}\n"),
    );
    assert!(
        trusted.status.success(),
        "{}",
        String::from_utf8_lossy(&trusted.stderr)
    );
    run(home.path(), &["lock"], "");

    // Inside the folder (and a subfolder), for the allowed environment: silent unlock.
    assert_eq!(
        json(&export_in(home.path(), "development", &project))["variables"]["KEY"],
        "dev"
    );
    run(home.path(), &["lock"], "");
    assert!(export_in(home.path(), "development", &nested)
        .status
        .success());
    run(home.path(), &["lock"], "");

    // Other environment, a sibling folder with the same prefix, or elsewhere: locked (exit 4).
    assert_eq!(
        export_in(home.path(), "production", &project).status.code(),
        Some(4)
    );
    let sibling = work.path().join("shop-other");
    assert_eq!(
        export_in(home.path(), "development", &sibling)
            .status
            .code(),
        Some(4)
    );
    assert_eq!(
        export_in(home.path(), "development", work.path())
            .status
            .code(),
        Some(4)
    );

    // Removing the folder closes it again, and the device does not fall back to "anywhere".
    assert!(run(
        home.path(),
        &["trust", "remove", project.to_str().unwrap()],
        ""
    )
    .status
    .success());
    assert_eq!(
        export_in(home.path(), "development", &project)
            .status
            .code(),
        Some(4)
    );
}
