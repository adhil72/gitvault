//! Collaborators: GitHub users invited to a vault's repository.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::CoreError;

/// What an invited user may do in the vault's repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum CollaboratorRole {
    /// Can clone and read the vault, cannot push changes.
    Read,
    /// Can also push secret changes.
    Write,
}

/// A GitHub username, checked the way GitHub checks it, so a typo never reaches the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubLogin(String);

impl GithubLogin {
    /// # Errors
    /// [`CoreError::Validation`] for anything GitHub would not accept as a username.
    pub fn parse(text: &str) -> Result<Self, CoreError> {
        let login = text.trim().trim_start_matches('@');
        let valid = (1..=39).contains(&login.len())
            && !login.starts_with('-')
            && !login.ends_with('-')
            && !login.contains("--")
            && login.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if valid {
            Ok(Self(login.to_owned()))
        } else {
            Err(CoreError::Validation(format!(
                "'{text}' is not a GitHub username"
            )))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::GithubLogin;

    #[test]
    fn accepts_github_usernames_and_strips_a_leading_at() {
        assert_eq!(GithubLogin::parse("octo-cat").unwrap().as_str(), "octo-cat");
        assert_eq!(GithubLogin::parse(" @alice ").unwrap().as_str(), "alice");
    }

    #[test]
    fn rejects_text_github_would_refuse() {
        for bad in [
            "",
            "-x",
            "x-",
            "a--b",
            "has space",
            "a/b",
            "émile",
            &"a".repeat(40),
        ] {
            assert!(
                GithubLogin::parse(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }
}
