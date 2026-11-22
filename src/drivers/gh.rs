use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use crate::{drivers::paths::ensure_private_dir, error::AppError};

/// Thin wrapper around the `gh` CLI. Never stores or inspects GitHub tokens.
pub struct GhDriver;

impl GhDriver {
    fn run(args: &[&str]) -> Result<Output, AppError> {
        Command::new("gh")
            .args(args)
            .output()
            .map_err(|e| AppError::Gh(format!("failed to execute gh: {e}")))
    }

    pub fn current_user() -> Result<String, AppError> {
        let out = Self::run(&["api", "user", "--jq", ".login"])?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
            return Err(AppError::Gh(format!(
                "could not resolve GitHub user via gh api: {err}"
            )));
        }
        let login = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if login.is_empty() {
            return Err(AppError::Gh("gh api user returned an empty login".into()));
        }
        Ok(login)
    }

    pub fn repo_exists(full_name: &str) -> Result<bool, AppError> {
        let out = Self::run(&["repo", "view", full_name, "--json", "name"])?;
        if out.status.success() {
            return Ok(true);
        }
        let stderr = String::from_utf8_lossy(&out.stderr).to_ascii_lowercase();
        let is_missing = ["could not resolve", "not found", "404", "does not exist"]
            .iter()
            .any(|needle| stderr.contains(needle));
        if is_missing {
            return Ok(false);
        }
        Err(AppError::Gh(format!(
            "gh repo view {full_name} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }

    pub fn create_private_repo(repo_name: &str) -> Result<(), AppError> {
        let out = Self::run(&[
            "repo",
            "create",
            repo_name,
            "--private",
            "--description",
            "Encrypted GitVault v2 secrets store",
        ])?;
        if !out.status.success() {
            return Err(AppError::Gh(format!(
                "gh repo create {repo_name} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(())
    }

    pub fn clone_repo(full_name: &str, target_dir: &Path) -> Result<(), AppError> {
        if let Some(parent) = target_dir.parent() {
            ensure_private_dir(parent)?;
        }
        let target_str = target_dir.to_string_lossy();
        let out = Self::run(&["repo", "clone", full_name, target_str.as_ref()])?;
        if !out.status.success() {
            return Err(AppError::Gh(format!(
                "gh repo clone {full_name} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        #[cfg(unix)]
        if target_dir.exists() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(target_dir, fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }

    pub fn setup_git_auth() -> Result<(), AppError> {
        let out = Self::run(&["auth", "setup-git"])?;
        if !out.status.success() {
            return Err(AppError::Gh(format!(
                "gh auth setup-git failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(())
    }
}
