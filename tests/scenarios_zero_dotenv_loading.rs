#![allow(clippy::unwrap_used)]

mod common;

use std::{path::Path, process::Command};

use common::{assert_no_dotenv, ScenarioHarness};

#[test]
fn scenario_load_into_child_process_and_exit_code_propagation() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "worker", "development", "master-pass-123");
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "QUEUE_URL=amqp://localhost"],
    );

    let ok = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["load", "--", "printenv", "QUEUE_URL"],
    );
    assert_eq!(
        String::from_utf8_lossy(&ok.stdout).trim(),
        "amqp://localhost"
    );
    assert_no_dotenv(&h.project_dir);

    // Child process failure exit code propagates cleanly
    h.run_err(
        &h.home1,
        &h.project_dir,
        &["load", "--", "sh", "-c", "exit 42"],
        42,
    );
}

#[test]
fn scenario_bash_shell_init_load_switch_and_unload() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "web", "development", "master-pass-123");
    h.run_ok(&h.home1, &h.project_dir, &["config", "set", "ENV_TAG=dev"]);
    h.run_ok(
        &h.home1,
        &h.project_dir,
        &["config", "set", "ENV_TAG=prod", "-e", "production"],
    );

    let bin_parent = Path::new(env!("CARGO_BIN_EXE_gitvault")).parent().unwrap();
    let script = r#"
eval "$(gitvault shell-init bash)"
gitvault load
printf "DEV:%s|%s\n" "$ENV_TAG" "$GITVAULT_ACTIVE_ENV"
gitvault load -e production
printf "PROD:%s|%s\n" "$ENV_TAG" "$GITVAULT_ACTIVE_ENV"
gitvault unload
printf "UNLOADED:%s|%s\n" "${ENV_TAG:-unset}" "${GITVAULT_ACTIVE_ENV:-unset}"
"#;
    let out = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(&h.project_dir)
        .env(
            "PATH",
            format!("{}:{}", bin_parent.display(), h.path_with_fake_gh()),
        )
        .env("GITVAULT_HOME", &h.home1)
        .env("FAKE_GH_REMOTE", &h.remote_bare)
        .env("FAKE_GH_STATE", &h.gh_state)
        .env("FAKE_GH_AUTH", "1")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("DEV:dev|development"));
    assert!(stdout.contains("PROD:prod|production"));
    assert!(stdout.contains("UNLOADED:unset|unset"));
    assert_no_dotenv(&h.project_dir);
}

#[test]
fn scenario_fish_and_posix_special_characters_safety() {
    let h = ScenarioHarness::new();
    h.init_project(&h.home1, "sec", "development", "master-pass-123");

    let tricky = "a'b $(touch /tmp/nope) `id` \"quoted\"";
    h.run_stdin(
        &h.home1,
        &h.project_dir,
        &["config", "set", "TRICKY", "--value-stdin"],
        &format!("{tricky}\n"),
    );

    let bin_parent = Path::new(env!("CARGO_BIN_EXE_gitvault")).parent().unwrap();
    let sh_out = Command::new("sh")
        .arg("-c")
        .arg(r#"eval "$(gitvault load --export)" && printf "%s" "$TRICKY""#)
        .current_dir(&h.project_dir)
        .env(
            "PATH",
            format!("{}:{}", bin_parent.display(), h.path_with_fake_gh()),
        )
        .env("GITVAULT_HOME", &h.home1)
        .env("FAKE_GH_REMOTE", &h.remote_bare)
        .env("FAKE_GH_STATE", &h.gh_state)
        .env("FAKE_GH_AUTH", "1")
        .output()
        .unwrap();
    assert!(sh_out.status.success());
    assert_eq!(String::from_utf8_lossy(&sh_out.stdout), tricky);

    let fish_export = h.run_ok(
        &h.home1,
        &h.project_dir,
        &["load", "--export", "--shell", "fish"],
    );
    let fish_txt = String::from_utf8_lossy(&fish_export.stdout);
    assert!(fish_txt.contains("set -gx TRICKY "));
}
