#![allow(clippy::unwrap_used)]

mod common;

use common::{assert_no_dotenv, ScenarioHarness};

#[test]
fn scenario_two_machines_collaborating_via_git_and_gh() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "team-api", "development", "shared-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "STRIPE_KEY=sk_from_m1"],
    );

    // Machine 2 clones vault store automatically via `gh` and reads secret
    let m2_read = h
        .cmd(&h.home2, &h.project_dir)
        .env("GITVAULT_PASSWORD", "shared-pass-123")
        .args(["--non-interactive", "load", "--", "printenv", "STRIPE_KEY"])
        .output()
        .unwrap();
    assert!(m2_read.status.success());
    assert_eq!(
        String::from_utf8_lossy(&m2_read.stdout).trim(),
        "sk_from_m1"
    );

    // Machine 2 adds a new secret and pushes it back to GitHub
    h.run_ok(
        &h.home2,
        &h.project_dir,
        &["config", "set", "SENTRY_DSN=https://sentry.local/1"],
    );

    // Machine 1 syncs and reads Machine 2's secret with zero .env files
    h.run_ok(&h.home1, &h.project_dir, &["sync"]);
    let m1_read = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["load", "--no-sync", "--", "printenv", "SENTRY_DSN"],
    );
    assert_eq!(
        String::from_utf8_lossy(&m1_read.stdout).trim(),
        "https://sentry.local/1"
    );
    assert_no_dotenv(&h.project_dir);
}

#[test]
fn scenario_sync_status_and_offline_commit_recovery() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "offline-app", "development", "master-pass-123");
    h.run_ok(&h.home1, &h.project_dir, &["config", "set", "A=1"]);

    // Simulate network outage by hiding the remote repo
    h.go_offline();
    let offline_set = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "OFFLINE_KEY=saved_offline"],
    );
    assert!(
        String::from_utf8_lossy(&offline_set.stderr).contains("sync deferred"),
        "expected offline warning"
    );

    // Local load still works offline
    let local_read = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["load", "--no-sync", "--", "printenv", "OFFLINE_KEY"],
    );
    assert_eq!(
        String::from_utf8_lossy(&local_read.stdout).trim(),
        "saved_offline"
    );

    // Restore remote: status shows ahead=1, then `sync` pushes to remote
    h.go_online();
    let status = h.run_ok(&h.home1, &h.project_dir, &["sync", "--status"]);
    assert!(String::from_utf8_lossy(&status.stdout).contains("Ahead:   1"));

    h.run_ok(&h.home1, &h.project_dir, &["sync"]);
    let m2_verify = h
        .cmd(&h.home2, &h.project_dir)
        .env("GITVAULT_PASSWORD", "master-pass-123")
        .args(["--non-interactive", "load", "--", "printenv", "OFFLINE_KEY"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&m2_verify.stdout).trim(),
        "saved_offline"
    );
}
