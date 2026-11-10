//! Running `git` and turning failures into typed errors.

use std::{
    path::Path,
    process::{Command, Output},
};

use base64::{engine::general_purpose::STANDARD, Engine};
use gitvault_core::{ports::Author, CoreError};

/// Extra settings for one git invocation.
#[derive(Default)]
pub struct Options<'a> {
    pub token: Option<&'a str>,
    pub author: Option<&'a Author>,
}

pub fn run(dir: &Path, args: &[&str], options: &Options<'_>) -> Result<Output, CoreError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "echo")
        .env("LC_ALL", "C");
    apply_author(&mut command, options.author);
    apply_token(&mut command, options.token);
    command.output().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CoreError::Git("git is not installed or not on PATH".into())
        } else {
            CoreError::Git(error.to_string())
        }
    })
}

fn apply_author(command: &mut Command, author: Option<&Author>) {
    let fallback = Author {
        name: "GitVault".into(),
        email: "gitvault@localhost".into(),
    };
    let author = author.unwrap_or(&fallback);
    command
        .env("GIT_AUTHOR_NAME", &author.name)
        .env("GIT_AUTHOR_EMAIL", &author.email)
        .env("GIT_COMMITTER_NAME", &author.name)
        .env("GIT_COMMITTER_EMAIL", &author.email);
}

fn apply_token(command: &mut Command, token: Option<&str>) {
    let Some(token) = token else { return };
    let basic = STANDARD.encode(format!("x-access-token:{token}"));
    command
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
        .env(
            "GIT_CONFIG_VALUE_0",
            format!("Authorization: Basic {basic}"),
        );
}

/// Runs git and returns trimmed stdout, or a typed error.
pub fn run_text(dir: &Path, args: &[&str], options: &Options<'_>) -> Result<String, CoreError> {
    let output = run(dir, args, options)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    Err(classify(&String::from_utf8_lossy(&output.stderr)))
}

/// Maps git's stderr to the most useful error type.
#[must_use]
pub fn classify(stderr: &str) -> CoreError {
    let text = stderr.trim();
    let lower = text.to_lowercase();
    let contains = |needles: &[&str]| needles.iter().any(|n| lower.contains(n));
    if contains(&[
        "could not resolve host",
        "unable to access",
        "failed to connect",
        "connection",
        "timed out",
        "network is unreachable",
    ]) {
        return CoreError::Network(first_line(text));
    }
    if contains(&[
        "authentication failed",
        "invalid username or password",
        "could not read username",
        "permission to",
        "403",
        "401",
    ]) {
        return CoreError::NotAuthenticated;
    }
    CoreError::Git(first_line(text))
}

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or("unknown git error")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::classify;
    use gitvault_core::ErrorCode;

    #[test]
    fn classifies_network_and_auth_failures() {
        assert_eq!(
            classify("fatal: unable to access 'https://x/': Could not resolve host").code(),
            ErrorCode::Network
        );
        assert_eq!(
            classify("remote: Invalid username or password.").code(),
            ErrorCode::NotAuthenticated
        );
        assert_eq!(
            classify("fatal: not a git repository").code(),
            ErrorCode::Internal
        );
    }
}
