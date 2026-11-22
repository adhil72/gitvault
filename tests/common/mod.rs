#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use tempfile::TempDir;

pub struct ScenarioHarness {
    pub root: TempDir,
    pub bin_dir: PathBuf,
    pub remote_bare: PathBuf,
    pub remote_offline: PathBuf,
    pub gh_state: PathBuf,
    pub home1: PathBuf,
    pub home2: PathBuf,
    pub project_dir: PathBuf,
}

impl ScenarioHarness {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let bin_dir = root.path().join("bin");
        let remote_bare = root.path().join("remote-gitvault-store.git");
        let remote_offline = root.path().join("remote-gitvault-store.offline");
        let gh_state = root.path().join("gh-created.flag");
        let home1 = root.path().join("home1");
        let home2 = root.path().join("home2");
        let project_dir = root.path().join("shop-api");

        for dir in [&bin_dir, &home1, &home2, &project_dir] {
            fs::create_dir_all(dir).unwrap();
        }
        write_fake_gh(&bin_dir.join("gh"));

        Self {
            root,
            bin_dir,
            remote_bare,
            remote_offline,
            gh_state,
            home1,
            home2,
            project_dir,
        }
    }

    pub fn path_with_fake_gh(&self) -> String {
        let base = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_owned());
        format!("{}:{base}", self.bin_dir.display())
    }

    pub fn cmd(&self, home: &Path, cwd: &Path) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_gitvault"));
        c.current_dir(cwd)
            .env("PATH", self.path_with_fake_gh())
            .env("GITVAULT_HOME", home)
            .env("FAKE_GH_REMOTE", &self.remote_bare)
            .env("FAKE_GH_STATE", &self.gh_state)
            .env("FAKE_GH_AUTH", "1")
            .env("NO_COLOR", "1");
        c
    }

    pub fn run_ok(&self, home: &Path, cwd: &Path, args: &[&str]) -> Output {
        let out = self.cmd(home, cwd).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "command {:?} failed (code {:?}): {}",
            args,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    pub fn run_err(&self, home: &Path, cwd: &Path, args: &[&str], expected_code: i32) -> Output {
        let out = self.cmd(home, cwd).args(args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(expected_code),
            "expected exit {expected_code} for {:?}, got {:?}: {}",
            args,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    pub fn run_stdin(&self, home: &Path, cwd: &Path, args: &[&str], input: &str) -> Output {
        let mut c = self.cmd(home, cwd);
        c.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = c.spawn().unwrap();
        if let Some(mut sin) = child.stdin.take() {
            sin.write_all(input.as_bytes()).unwrap();
        }
        child.wait_with_output().unwrap()
    }

    pub fn init_project(&self, home: &Path, project: &str, env: &str, password: &str) -> Output {
        let out = self.run_stdin(
            home,
            &self.project_dir,
            &[
                "--non-interactive",
                "--password-stdin",
                "init",
                "-p",
                project,
                "-e",
                env,
            ],
            &format!("{password}\n"),
        );
        assert!(
            out.status.success(),
            "init failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_no_dotenv(&self.project_dir);
        out
    }

    pub fn go_offline(&self) {
        fs::rename(&self.remote_bare, &self.remote_offline).unwrap();
    }

    pub fn go_online(&self) {
        fs::rename(&self.remote_offline, &self.remote_bare).unwrap();
    }
}

pub fn assert_no_dotenv(dir: &Path) {
    assert!(
        !dir.join(".env").exists(),
        "forbidden plaintext .env found in {}",
        dir.display()
    );
}

pub fn extract_recovery_key(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .find(|line| line.split('-').count() == 8 && line.len() == 39)
        .expect("recovery key should appear in stderr on initial setup")
        .to_owned()
}

fn write_fake_gh(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let script = r#"#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  --version) echo "gh version 2.46.0 (test)"; exit 0 ;;
  auth)
    case "${2:-}" in
      status)
        if [ "${FAKE_GH_AUTH:-1}" = "1" ]; then
          echo "Logged in to github.com account octocat"; exit 0
        else
          echo "You are not logged into any GitHub hosts." >&2; exit 1
        fi ;;
      setup-git) exit 0 ;;
    esac ;;
  api)
    if [ "${2:-}" = "user" ]; then echo "octocat"; exit 0; fi ;;
  repo)
    case "${2:-}" in
      view)
        if [ -f "$FAKE_GH_STATE" ]; then echo '{"name":"gitvault-store"}'; exit 0
        else echo "GraphQL: Could not resolve to a Repository" >&2; exit 1; fi ;;
      create)
        git init --bare -b main "$FAKE_GH_REMOTE" >/dev/null 2>&1
        touch "$FAKE_GH_STATE"; exit 0 ;;
      clone)
        git clone "$FAKE_GH_REMOTE" "${4:?missing target}" >/dev/null 2>&1; exit 0 ;;
    esac ;;
esac
echo "unexpected fake gh call: $*" >&2; exit 1
"#;
    fs::write(path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
