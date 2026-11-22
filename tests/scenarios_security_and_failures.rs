#![allow(clippy::unwrap_used)]

mod common;

use std::{fs, path::PathBuf, process::Command};

use common::ScenarioHarness;

#[test]
fn scenario_prerequisite_and_auth_failures() {
    let h = ScenarioHarness::new();

    // Missing `gh` binary -> exit 2
    let only_git = h.root.path().join("only-git");
    fs::create_dir_all(&only_git).unwrap();
    let real_git = Command::new("which").arg("git").output().unwrap().stdout;
    let real_git_path = PathBuf::from(String::from_utf8_lossy(&real_git).trim());
    std::os::unix::fs::symlink(real_git_path, only_git.join("git")).unwrap();

    let missing_gh = Command::new(env!("CARGO_BIN_EXE_gitvault"))
        .current_dir(&h.project_dir)
        .env("PATH", &only_git)
        .env("GITVAULT_HOME", &h.home1)
        .args(["--non-interactive", "init", "-p", "demo", "-e", "dev"])
        .output()
        .unwrap();
    assert_eq!(missing_gh.status.code(), Some(2));

    // Unauthenticated `gh` -> exit 3
    let unauth = h
        .cmd(&h.home1, &h.project_dir)
        .env("FAKE_GH_AUTH", "0")
        .args(["--non-interactive", "init", "-p", "demo", "-e", "dev"])
        .output()
        .unwrap();
    assert_eq!(unauth.status.code(), Some(3));

    // Locked vault in non-interactive mode -> exit 4
    h.init_project(&h.home1, "sec-app", "development", "master-pass-123");
    h.run_ok(&h.home1, &h.project_dir, &["lock"]);
    h.run_err(
        &h.home1,
        &h.project_dir,
        &["--non-interactive", "load", "--json"],
        4,
    );

    // Wrong password -> exit 5
    let wrong_pw = h
        .cmd(&h.home1, &h.project_dir)
        .env("GITVAULT_PASSWORD", "wrong-pass")
        .args(["--non-interactive", "unlock"])
        .output()
        .unwrap();
    assert_eq!(wrong_pw.status.code(), Some(5));
}

#[test]
fn scenario_validation_blocks_path_traversal_and_bad_keys() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "valid-app", "development", "master-pass-123");

    for bad_slug in ["../evil", "a/b", "has space"] {
        h.run_err(
            &h.home1,
            &h.project_dir,
            &["--non-interactive", "init", "-p", bad_slug, "-e", "dev"],
            7,
        );
    }
    for bad_key in ["123KEY=x", "BAD-KEY=x", "KEY;rm=x"] {
        h.run_err(&h.home1, &h.project_dir, &["config", "set", bad_key], 7);
    }
    h.run_err(&h.home1, &h.project_dir, &["config", "get", "MISSING"], 6);
}

#[test]
fn scenario_aead_detects_cross_env_swap_and_ciphertext_tampering() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "tamper-app", "development", "master-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "FOO=dev", "-e", "development"],
    );
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "FOO=prod", "-e", "production"],
    );

    let envs_dir = h.home1.join("store/projects/tamper-app/envs");
    fs::copy(
        envs_dir.join("development.enc"),
        envs_dir.join("production.enc"),
    )
    .unwrap();
    h.run_err(
        &h.home1,
        &h.project_dir,
        &["load", "-e", "production", "--no-sync", "--json"],
        8,
    );

    corrupt_ciphertext(&envs_dir.join("development.enc"));
    h.run_err(
        &h.home1,
        &h.project_dir,
        &["load", "-e", "development", "--no-sync", "--json"],
        8,
    );
}

fn corrupt_ciphertext(path: &std::path::Path) {
    let mut blob: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let mut ct = blob["ciphertext"].as_str().unwrap().as_bytes().to_vec();
    ct[0] = if ct[0] == b'A' { b'B' } else { b'A' };
    blob["ciphertext"] = serde_json::Value::String(String::from_utf8(ct).unwrap());
    fs::write(path, serde_json::to_string(&blob).unwrap()).unwrap();
}

#[cfg(unix)]
#[test]
fn scenario_unix_permissions_enforced() {
    use std::os::unix::fs::PermissionsExt;

    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "perm-app", "development", "master-pass-123");

    let store_mode = fs::metadata(h.home1.join("store"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let session_mode = fs::metadata(h.home1.join("session.key"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(store_mode, 0o700);
    assert_eq!(session_mode, 0o600);
}
