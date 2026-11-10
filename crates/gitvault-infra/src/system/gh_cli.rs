use std::process::Command;

use gitvault_core::{error::CoreError, ports::GhCli};

/// Runs `gh auth token`. The token is returned to the caller and never logged.
pub struct GhCliProcess;

impl GhCli for GhCliProcess {
    fn token(&self) -> Result<String, CoreError> {
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output()
            .map_err(|_| {
                CoreError::Validation(
                    "The GitHub CLI (gh) was not found. Install it from https://cli.github.com"
                        .into(),
                )
            })?;
        let token = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if !output.status.success() || token.is_empty() {
            return Err(CoreError::Validation(
                "The GitHub CLI is not signed in. Run `gh auth login` first.".into(),
            ));
        }
        Ok(token)
    }
}
