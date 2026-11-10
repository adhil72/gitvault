use serde::{Deserialize, Serialize};

/// The signed-in GitHub user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubAccount {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub avatar_url: String,
}

impl GithubAccount {
    /// The no-reply address GitHub associates with this account, used for commit authorship.
    #[must_use]
    pub fn commit_email(&self) -> String {
        format!("{}+{}@users.noreply.github.com", self.id, self.login)
    }
}
