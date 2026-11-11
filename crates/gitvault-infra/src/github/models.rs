//! JSON shapes returned by GitHub, and their conversion to core types.

use gitvault_core::{
    domain::{account::GithubAccount, collaborator::CollaboratorRole},
    ports::{Collaborator, DeviceCode, DevicePoll, RemoteRepo, RepoAccess},
};
use serde::Deserialize;

pub const VAULT_TOPIC: &str = "gitvault-vault";

#[derive(Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    #[serde(default = "default_interval")]
    pub interval: u32,
}

const fn default_interval() -> u32 {
    5
}

impl From<DeviceCodeResponse> for DeviceCode {
    fn from(r: DeviceCodeResponse) -> Self {
        Self {
            device_code: r.device_code,
            user_code: r.user_code,
            verification_uri: r.verification_uri,
            expires_in: r.expires_in,
            interval: r.interval,
        }
    }
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub error: Option<String>,
}

/// Maps the token endpoint's answer to a poll result.
pub fn to_poll(response: TokenResponse) -> DevicePoll {
    if let Some(token) = response.access_token {
        return DevicePoll::Authorized { token };
    }
    match response.error.as_deref() {
        Some("authorization_pending") => DevicePoll::Pending,
        Some("slow_down") => DevicePoll::SlowDown,
        Some("access_denied") => DevicePoll::Denied,
        _ => DevicePoll::Expired,
    }
}

#[derive(Deserialize)]
pub struct UserResponse {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub avatar_url: String,
}

impl From<UserResponse> for GithubAccount {
    fn from(u: UserResponse) -> Self {
        Self {
            login: u.login,
            id: u.id,
            name: u.name,
            avatar_url: u.avatar_url,
        }
    }
}

#[derive(Deserialize)]
pub struct RepoResponse {
    pub full_name: String,
    pub clone_url: String,
    pub private: bool,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub permissions: Option<Permissions>,
}

#[derive(Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub push: bool,
}

impl RepoResponse {
    pub fn is_vault(&self) -> bool {
        self.topics.iter().any(|t| t == VAULT_TOPIC)
    }

    /// A visible repository is at least readable; `push` upgrades it to write access.
    pub fn access(&self) -> RepoAccess {
        match &self.permissions {
            Some(p) if p.push => RepoAccess::Write,
            _ => RepoAccess::Read,
        }
    }
}

impl From<RepoResponse> for RemoteRepo {
    fn from(r: RepoResponse) -> Self {
        Self {
            full_name: r.full_name,
            clone_url: r.clone_url,
            private: r.private,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CollaboratorResponse {
    pub login: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub permissions: Option<Permissions>,
}

#[derive(Deserialize)]
pub struct Invitee {
    pub login: String,
    #[serde(default)]
    pub avatar_url: String,
}

/// A pending repository invitation. `permissions` is GitHub's role name, e.g. `read`.
#[derive(Deserialize)]
pub struct InvitationResponse {
    pub invitee: Option<Invitee>,
    #[serde(default)]
    pub permissions: String,
}

impl From<CollaboratorResponse> for Collaborator {
    fn from(c: CollaboratorResponse) -> Self {
        let push = c.permissions.is_some_and(|p| p.push);
        Self {
            login: c.login,
            avatar_url: c.avatar_url,
            role: if push {
                CollaboratorRole::Write
            } else {
                CollaboratorRole::Read
            },
            pending: false,
        }
    }
}

impl InvitationResponse {
    /// `None` for invitations GitHub sent to an address rather than a user.
    pub fn into_collaborator(self) -> Option<Collaborator> {
        let invitee = self.invitee?;
        let role = match self.permissions.as_str() {
            "read" | "triage" => CollaboratorRole::Read,
            _ => CollaboratorRole::Write,
        };
        Some(Collaborator {
            login: invitee.login,
            avatar_url: invitee.avatar_url,
            role,
            pending: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(access: Option<&str>, error: Option<&str>) -> TokenResponse {
        TokenResponse {
            access_token: access.map(str::to_owned),
            error: error.map(str::to_owned),
        }
    }

    #[test]
    fn maps_device_flow_answers() {
        assert_eq!(
            to_poll(token(Some("t"), None)),
            DevicePoll::Authorized { token: "t".into() }
        );
        assert_eq!(
            to_poll(token(None, Some("authorization_pending"))),
            DevicePoll::Pending
        );
        assert_eq!(
            to_poll(token(None, Some("slow_down"))),
            DevicePoll::SlowDown
        );
        assert_eq!(
            to_poll(token(None, Some("access_denied"))),
            DevicePoll::Denied
        );
        assert_eq!(
            to_poll(token(None, Some("expired_token"))),
            DevicePoll::Expired
        );
    }

    #[test]
    fn detects_vault_repositories_by_topic() {
        let repo: RepoResponse = serde_json::from_str(
            r#"{"full_name":"o/r","clone_url":"https://github.com/o/r.git","private":true,"topics":["gitvault-vault"]}"#,
        )
        .unwrap();
        assert!(repo.is_vault());
        let other: RepoResponse =
            serde_json::from_str(r#"{"full_name":"o/x","clone_url":"u","private":false}"#).unwrap();
        assert!(!other.is_vault());
    }
}
