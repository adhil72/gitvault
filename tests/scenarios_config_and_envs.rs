#![allow(clippy::unwrap_used)]

mod common;

use std::fs;

use common::{assert_no_dotenv, ScenarioHarness};

fn assert_var_eq(h: &ScenarioHarness, env: &str, key: &str, expected: &str) {
    let out = h.run_ok(&h.home1, &h.project_dir, &["config", "get", key, "-e", env]);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), expected);
}

#[test]
fn scenario_multi_env_lifecycle_create_clone_delete() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "billing-api", "development", "master-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "DB_HOST=dev.internal", "LOG=debug"],
    );
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "env", "create", "preview"],
    );
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "env", "clone", "development", "preview"],
    );
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "DB_HOST=preview.internal", "-e", "preview"],
    );

    assert_var_eq(&h, "development", "DB_HOST", "dev.internal");
    assert_var_eq(&h, "preview", "DB_HOST", "preview.internal");

    let envs = h.run_ok(&h.home1, &h.project_dir, &["config", "env", "list"]);
    assert!(String::from_utf8_lossy(&envs.stdout).contains("preview\t2 variable(s)"));

    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "env", "delete", "preview", "--yes"],
    );
    let after = h.run_ok(&h.home1, &h.project_dir, &["config", "env", "list"]);
    assert!(!String::from_utf8_lossy(&after.stdout).contains("preview"));
}

#[test]
fn scenario_legacy_dotenv_migration_overwrite_and_shred() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "legacy-app", "development", "master-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "KEEP_ME=original"],
    );

    let legacy_path = h.project_dir.join(".env");
    let dotenv_content = concat!(
        "# Existing .env to eliminate\n",
        "KEEP_ME=new_value\n",
        "export JWT_SECRET=\"line1\\nline2\"\n",
        "PORT=4000 # trailing comment\n",
    );
    fs::write(&legacy_path, dotenv_content).unwrap();

    // Without --overwrite: existing KEEP_ME stays untouched
    h.run_ok(&h.home1, &h.project_dir, &["config", "import", ".env"]);
    let keep = h.run_ok(&h.home1, &h.project_dir, &["config", "get", "KEEP_ME"]);
    assert_eq!(String::from_utf8_lossy(&keep.stdout).trim(), "original");
    assert!(legacy_path.is_file());

    // With --overwrite --delete-source: updates KEEP_ME and deletes plaintext .env
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "import", ".env", "--overwrite", "--delete-source"],
    );
    assert_no_dotenv(&h.project_dir);

    let updated = h.run_ok(&h.home1, &h.project_dir, &["config", "get", "KEEP_ME"]);
    assert_eq!(String::from_utf8_lossy(&updated.stdout).trim(), "new_value");
    let jwt = h.run_ok(&h.home1, &h.project_dir, &["config", "get", "JWT_SECRET"]);
    assert_eq!(String::from_utf8_lossy(&jwt.stdout).trim(), "line1\nline2");
}
