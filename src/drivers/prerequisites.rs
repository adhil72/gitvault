use std::process::Command;

use crate::{
    drivers::{skip_gh_check, GhDriver},
    error::AppError,
};

/// Diagnostic status for `gitvault doctor`.
#[derive(Debug)]
pub struct PrereqReport {
    pub git_version: Option<String>,
    pub gh_version: Option<String>,
    pub gh_auth_ok: bool,
    pub gh_user: Option<String>,
    pub gh_auth_message: Option<String>,
}

/// Verifies `git`, `gh`, and `gh auth status` before running vault operations.
pub struct Prerequisites;

impl Prerequisites {
    pub fn ensure_ready() -> Result<(), AppError> {
        if !command_succeeds("git", &["--version"]) {
            return Err(AppError::PrerequisiteMissing {
                tool: "git".into(),
                hint: "Install Git from https://git-scm.com/downloads and retry.".into(),
            });
        }
        if skip_gh_check() {
            return Ok(());
        }
        if !command_succeeds("gh", &["--version"]) {
            return Err(AppError::PrerequisiteMissing {
                tool: "gh".into(),
                hint: "Install GitHub CLI from https://cli.github.com and run 'gh auth login'."
                    .into(),
            });
        }
        if !command_succeeds("gh", &["auth", "status"]) {
            return Err(AppError::NotAuthenticated {
                hint: "Run 'gh auth login' first. GitVault uses gh directly for all GitHub access."
                    .into(),
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn diagnose() -> PrereqReport {
        let git_version = probe_version("git");
        let gh_version = probe_version("gh");
        let (gh_auth_ok, gh_auth_message) = probe_gh_auth();
        let gh_user = if gh_auth_ok {
            GhDriver::current_user().ok()
        } else {
            None
        };
        PrereqReport {
            git_version,
            gh_version,
            gh_auth_ok,
            gh_user,
            gh_auth_message,
        }
    }
}

fn command_succeeds(bin: &str, args: &[&str]) -> bool {
    Command::new(bin)
        .args(args)
        .output()
        .is_ok_and(|out| out.status.success())
}

fn probe_version(bin: &str) -> Option<String> {
    let out = Command::new(bin).arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(|s| s.trim().to_owned())
}

fn probe_gh_auth() -> (bool, Option<String>) {
    match Command::new("gh").args(["auth", "status"]).output() {
        Ok(out) if out.status.success() => (true, None),
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
            let msg = if err.is_empty() {
                String::from_utf8_lossy(&out.stdout).trim().to_owned()
            } else {
                err
            };
            (false, Some(msg))
        }
        Err(e) => (false, Some(e.to_string())),
    }
}
