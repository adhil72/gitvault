#![allow(clippy::unwrap_used)]

mod common;

use std::fs;

use common::{assert_no_dotenv, extract_recovery_key, ScenarioHarness};

#[test]
fn scenario_new_developer_onboarding_and_nested_dir_resolution() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "shop-api", "development", "master-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "PORT=8080", "SERVICE_NAME=shop"],
    );

    let nested = h.project_dir.join("packages/backend/src");
    fs::create_dir_all(&nested).unwrap();

    let loaded = h.run_ok(&h.home1, &nested, &["load", "--", "printenv", "PORT"]);
    assert_eq!(String::from_utf8_lossy(&loaded.stdout).trim(), "8080");
    assert_no_dotenv(&h.project_dir);
    assert_no_dotenv(&nested);

    let projects = h.run_ok(&h.home1, &nested, &["projects", "--json"]);
    let parsed: serde_json::Value = serde_json::from_slice(&projects.stdout).unwrap();
    assert_eq!(parsed[0]["name"], "shop-api");

    let doc = h.run_ok(&h.home1, &nested, &["doctor", "--json"]);
    let doc_json: serde_json::Value = serde_json::from_slice(&doc.stdout).unwrap();
    assert_eq!(doc_json["gh_authenticated"], true);
    assert_eq!(doc_json["vault_initialized"], true);
    assert_eq!(doc_json["vault_unlocked"], true);
    assert_eq!(doc_json["project_config"]["project"], "shop-api");
}

#[test]
fn scenario_interactive_init_and_config_menus() {
    let h = ScenarioHarness::new();

    // Master password + confirm, "+ Create new project" (1), accept folder name, default env (1)
    let init_inputs = "master-pass-123\nmaster-pass-123\n1\n\n1\n";
    let init_out = h.run_stdin(&h.home1, &h.project_dir, &["init"], init_inputs);
    assert!(init_out.status.success());
    assert!(h.project_dir.join("gitvault.json").is_file());
    assert_no_dotenv(&h.project_dir);

    // Config variables (1) -> staging (2) -> Add variable (1) -> REDIS_URL -> value -> Back (6) -> Exit (6)
    let cfg_inputs = "1\n2\n1\nREDIS_URL\nredis://localhost:6379\n6\n6\n";
    let cfg_out = h.run_stdin(&h.home1, &h.project_dir, &["config"], cfg_inputs);
    assert!(cfg_out.status.success());

    let loaded = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["load", "-e", "staging", "--", "printenv", "REDIS_URL"],
    );
    assert_eq!(
        String::from_utf8_lossy(&loaded.stdout).trim(),
        "redis://localhost:6379"
    );
}

#[test]
fn scenario_forgot_password_recovery_and_password_rotation() {
    let h = ScenarioHarness::new();
    let init_out = h.init_project(&h.home1, "shop-api", "development", "old-pass-123");
    let recovery_key = extract_recovery_key(&String::from_utf8_lossy(&init_out.stderr));

    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "STRIPE=sk_123"],
    );
    h.run_ok(&h.home1, &h.project_dir, &["lock"]);

    // Unlock using recovery key without knowing master password
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["unlock", "--recovery-key", &recovery_key],
    );

    // Rotate master password
    let rotate_out = h.run_stdin(
        &h.home1,
        &h.project_dir,
        &["unlock", "--change-password"],
        "old-pass-123\nnew-pass-456\nnew-pass-456\n",
    );
    assert!(rotate_out.status.success());

    // Old password must fail after rotation; new password unlocks existing secrets
    h.run_ok(&h.home1, &h.project_dir, &["lock"]);
    let bad = h
        .cmd(&h.home1, &h.project_dir)
        .env("GITVAULT_PASSWORD", "old-pass-123")
        .args(["--non-interactive", "unlock"])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(5));

    let good = h
        .cmd(&h.home1, &h.project_dir)
        .env("GITVAULT_PASSWORD", "new-pass-456")
        .args(["--non-interactive", "load", "--", "printenv", "STRIPE"])
        .output()
        .unwrap();
    assert!(good.status.success());
    assert_eq!(String::from_utf8_lossy(&good.stdout).trim(), "sk_123");
}
