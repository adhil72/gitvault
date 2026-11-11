use std::process::{Command, Output};

use gitvault_core::{
    error::CoreError,
    ports::{GhCli, GhCliAccount},
};

const HOST: &str = "github.com";

/// Runs `gh`. Tokens are returned to the caller and never logged.
pub struct GhCliProcess;

fn run(args: &[&str]) -> Result<Output, CoreError> {
    Command::new("gh").args(args).output().map_err(|_| {
        CoreError::Validation(
            "The GitHub CLI (gh) was not found. Install it from https://cli.github.com".into(),
        )
    })
}

fn read_token(output: &Output) -> Result<String, CoreError> {
    let token = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || token.is_empty() {
        return Err(CoreError::Validation(
            "The GitHub CLI is not signed in. Run `gh auth login` first.".into(),
        ));
    }
    Ok(token)
}

/// Parses the human-readable `gh auth status` output (older `gh` versions have no JSON mode).
fn parse_accounts(status: &str) -> Vec<GhCliAccount> {
    let mut accounts: Vec<GhCliAccount> = Vec::new();
    for line in status.lines().map(str::trim) {
        if let Some(login) = account_on_line(line) {
            accounts.push(GhCliAccount {
                login,
                active: false,
            });
        } else if line.starts_with("- Active account:") {
            if let Some(last) = accounts.last_mut() {
                last.active = line.ends_with("true");
            }
        }
    }
    accounts
}

fn account_on_line(line: &str) -> Option<String> {
    let rest = line.split(" account ").nth(1)?;
    if !line.contains(&format!("Logged in to {HOST}")) {
        return None;
    }
    rest.split_whitespace().next().map(str::to_owned)
}

impl GhCli for GhCliProcess {
    fn token(&self) -> Result<String, CoreError> {
        read_token(&run(&["auth", "token", "--hostname", HOST])?)
    }

    fn accounts(&self) -> Vec<GhCliAccount> {
        // Older versions print the status to stderr, newer ones to stdout.
        run(&["auth", "status", "--hostname", HOST])
            .map(|out| {
                let text = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                parse_accounts(&text)
            })
            .unwrap_or_default()
    }

    fn token_for(&self, login: &str) -> Result<String, CoreError> {
        read_token(&run(&[
            "auth",
            "token",
            "--hostname",
            HOST,
            "--user",
            login,
        ])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = "github.com\n  ✓ Logged in to github.com account one (keyring)\n  - Active account: true\n  - Token: gho_***\n\n  ✓ Logged in to github.com account two (keyring)\n  - Active account: false\n";

    #[test]
    fn parses_accounts_and_the_active_one() {
        let accounts = parse_accounts(STATUS);
        assert_eq!(accounts.len(), 2);
        assert_eq!(
            (accounts[0].login.as_str(), accounts[0].active),
            ("one", true)
        );
        assert_eq!(
            (accounts[1].login.as_str(), accounts[1].active),
            ("two", false)
        );
    }

    #[test]
    fn signed_out_output_has_no_accounts() {
        assert!(parse_accounts("You are not logged into any GitHub hosts.").is_empty());
    }
}
