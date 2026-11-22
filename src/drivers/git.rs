use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use crate::{
    drivers::{paths::ensure_private_dir, skip_gh_check, GhDriver},
    error::AppError,
};

const GIT_ATTRIBUTES: &str = "* binary\n";

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub remote_url: Option<String>,
    pub dirty_files: usize,
    pub ahead: u32,
    pub behind: u32,
}

/// Thin wrapper around the `git` CLI operating on the hidden vault store.
pub struct GitDriver;

impl GitDriver {
    fn run(dir: &Path, args: &[&str]) -> Result<Output, AppError> {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .map_err(|e| AppError::Git(format!("failed to run git: {e}")))
    }

    fn run_ok(dir: &Path, args: &[&str]) -> Result<String, AppError> {
        let out = Self::run(dir, args)?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
            let msg = if err.is_empty() {
                String::from_utf8_lossy(&out.stdout).trim().to_owned()
            } else {
                err
            };
            return Err(AppError::Git(msg));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }

    pub fn ensure_initialized(dir: &Path) -> Result<(), AppError> {
        ensure_private_dir(dir)?;
        if !dir.join(".git").exists() {
            Self::run_ok(dir, &["init", "-b", "main"])?;
        } else if !Self::has_commits(dir) {
            let _ = Self::run(dir, &["checkout", "-B", "main"]);
        }
        let gitattributes = dir.join(".gitattributes");
        if !gitattributes.exists() {
            fs::write(&gitattributes, GIT_ATTRIBUTES)?;
        }
        Ok(())
    }

    fn ensure_local_identity(dir: &Path) -> Result<(), AppError> {
        let has_name = Self::run_ok(dir, &["config", "user.name"]).is_ok_and(|s| !s.is_empty());
        let has_email = Self::run_ok(dir, &["config", "user.email"]).is_ok_and(|s| !s.is_empty());

        if !has_name {
            let login = if skip_gh_check() {
                "gitvault".to_owned()
            } else {
                GhDriver::current_user().unwrap_or_else(|_| "gitvault".to_owned())
            };
            Self::run_ok(dir, &["config", "--local", "user.name", &login])?;
        }
        if !has_email {
            Self::run_ok(
                dir,
                &[
                    "config",
                    "--local",
                    "user.email",
                    "gitvault@users.noreply.github.com",
                ],
            )?;
        }
        Ok(())
    }

    #[must_use]
    pub fn has_commits(dir: &Path) -> bool {
        Self::run(dir, &["rev-parse", "--verify", "HEAD"]).is_ok_and(|o| o.status.success())
    }

    #[must_use]
    pub fn remote_url(dir: &Path) -> Option<String> {
        Self::run_ok(dir, &["remote", "get-url", "origin"])
            .ok()
            .filter(|s| !s.is_empty())
    }

    #[must_use]
    pub fn has_remote(dir: &Path) -> bool {
        Self::remote_url(dir).is_some()
    }

    #[must_use]
    pub fn current_branch(dir: &Path) -> String {
        Self::run_ok(dir, &["symbolic-ref", "--short", "HEAD"])
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "main".to_owned())
    }

    fn remote_branch_exists(dir: &Path, branch: &str) -> bool {
        let spec = format!("origin/{branch}");
        Self::run(dir, &["rev-parse", "--verify", &spec]).is_ok_and(|o| o.status.success())
    }

    fn count_range(dir: &Path, range: &str) -> u32 {
        Self::run_ok(dir, &["rev-list", "--count", range])
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    pub fn commit_all(dir: &Path, message: &str) -> Result<bool, AppError> {
        Self::run_ok(dir, &["add", "-A"])?;
        if Self::run_ok(dir, &["status", "--porcelain"])?.is_empty() {
            return Ok(false);
        }
        Self::ensure_local_identity(dir)?;
        Self::run_ok(dir, &["commit", "-q", "-m", message])?;
        Ok(true)
    }

    pub fn pull_rebase(dir: &Path) -> Result<bool, AppError> {
        if !Self::has_remote(dir) {
            return Ok(false);
        }
        Self::run_ok(dir, &["fetch", "-q", "origin"])?;
        let branch = Self::current_branch(dir);
        if !Self::remote_branch_exists(dir, &branch) {
            return Ok(false);
        }
        if !Self::has_commits(dir) {
            Self::run_ok(
                dir,
                &["checkout", "-B", &branch, &format!("origin/{branch}")],
            )?;
            return Ok(true);
        }
        if Self::count_range(dir, &format!("HEAD..origin/{branch}")) == 0 {
            return Ok(false);
        }
        let res = Self::run(dir, &["pull", "--rebase", "-q", "origin", &branch])?;
        if !res.status.success() {
            let _ = Self::run(dir, &["rebase", "--abort"]);
            return Err(AppError::Git(format!(
                "rebase conflict while syncing with origin/{branch}: {}",
                String::from_utf8_lossy(&res.stderr).trim()
            )));
        }
        Ok(true)
    }

    pub fn push(dir: &Path) -> Result<bool, AppError> {
        if !Self::has_remote(dir) || !Self::has_commits(dir) {
            return Ok(false);
        }
        let branch = Self::current_branch(dir);
        let remote_exists = Self::remote_branch_exists(dir, &branch);
        let ahead = if remote_exists {
            Self::count_range(dir, &format!("origin/{branch}..HEAD"))
        } else {
            Self::count_range(dir, "HEAD")
        };
        if ahead == 0 && remote_exists {
            return Ok(false);
        }
        Self::run_ok(dir, &["push", "-q", "-u", "origin", &branch])?;
        Ok(true)
    }

    pub fn status(dir: &Path) -> Result<GitStatus, AppError> {
        let branch = Self::current_branch(dir);
        let remote_url = Self::remote_url(dir);
        let dirty_files = Self::run_ok(dir, &["status", "--porcelain"])
            .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
            .unwrap_or(0);
        let (ahead, behind) = if remote_url.is_some() && Self::remote_branch_exists(dir, &branch) {
            (
                Self::count_range(dir, &format!("origin/{branch}..HEAD")),
                Self::count_range(dir, &format!("HEAD..origin/{branch}")),
            )
        } else if Self::has_commits(dir) {
            (Self::count_range(dir, "HEAD"), 0)
        } else {
            (0, 0)
        };
        Ok(GitStatus {
            branch,
            remote_url,
            dirty_files,
            ahead,
            behind,
        })
    }
}
