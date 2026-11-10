use std::time::Duration;

use gitvault_core::{
    domain::account::GithubAccount,
    ports::{DeviceCode, DevicePoll, GitHost, RemoteRepo, RepoAccess},
    CoreError,
};
use reqwest::{blocking::Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use super::models::{
    to_poll, DeviceCodeResponse, RepoResponse, TokenResponse, UserResponse, VAULT_TOPIC,
};

const API: &str = "https://api.github.com";
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const REPOS_PER_PAGE: u32 = 100;
const MAX_PAGES: u32 = 5;
/// `repo` lets GitVault create a private vault repository and push to it.
const SCOPES: &str = "repo read:user";

/// [`GitHost`] backed by the GitHub REST API.
pub struct GithubHttp {
    client: Client,
}

impl GithubHttp {
    /// # Errors
    /// Fails if the HTTP client cannot be built.
    pub fn new() -> Result<Self, CoreError> {
        let client = Client::builder()
            .user_agent("gitvault")
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(Self { client })
    }

    fn get<T: DeserializeOwned>(&self, url: &str, token: &str) -> Result<T, CoreError> {
        let response = self
            .client
            .get(url)
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(network)?;
        check(response.status())?;
        response.json().map_err(network)
    }

    fn post_form<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &[(&str, &str)],
    ) -> Result<T, CoreError> {
        let response = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .form(form)
            .send()
            .map_err(network)?;
        if response.status() == StatusCode::BAD_REQUEST {
            // GitHub explains OAuth setup problems (e.g. device flow disabled) in the body.
            let body: Value = response.json().map_err(network)?;
            let reason = body["error_description"].as_str().unwrap_or("bad request");
            return Err(CoreError::Validation(format!("GitHub: {reason}")));
        }
        check(response.status())?;
        response.json().map_err(network)
    }
}

fn network(error: reqwest::Error) -> CoreError {
    CoreError::Network(error.without_url().to_string())
}

fn check(status: StatusCode) -> Result<(), CoreError> {
    match status {
        s if s.is_success() => Ok(()),
        StatusCode::UNAUTHORIZED => Err(CoreError::NotAuthenticated),
        StatusCode::FORBIDDEN => Err(CoreError::PermissionDenied(
            "GitHub refused the request (token scope or rate limit)".into(),
        )),
        StatusCode::UNPROCESSABLE_ENTITY => Err(CoreError::Conflict(
            "GitHub rejected the request (name already taken?)".into(),
        )),
        other => Err(CoreError::Network(format!("GitHub answered {other}"))),
    }
}

impl GitHost for GithubHttp {
    fn device_code(&self, client_id: &str) -> Result<DeviceCode, CoreError> {
        let response: DeviceCodeResponse = self.post_form(
            DEVICE_CODE_URL,
            &[("client_id", client_id), ("scope", SCOPES)],
        )?;
        Ok(response.into())
    }

    fn poll_device_token(
        &self,
        client_id: &str,
        device_code: &str,
    ) -> Result<DevicePoll, CoreError> {
        let response: TokenResponse = self.post_form(
            ACCESS_TOKEN_URL,
            &[
                ("client_id", client_id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ],
        )?;
        Ok(to_poll(response))
    }

    fn current_user(&self, token: &str) -> Result<GithubAccount, CoreError> {
        let user: UserResponse = self.get(&format!("{API}/user"), token)?;
        Ok(user.into())
    }

    fn find_vault_repos(&self, token: &str) -> Result<Vec<RemoteRepo>, CoreError> {
        let mut found = Vec::new();
        for page in 1..=MAX_PAGES {
            let url = format!(
                "{API}/user/repos?affiliation=owner,collaborator,organization_member&sort=updated&per_page={REPOS_PER_PAGE}&page={page}"
            );
            let repos: Vec<RepoResponse> = self.get(&url, token)?;
            let last_page = repos.len() < REPOS_PER_PAGE as usize;
            found.extend(
                repos
                    .into_iter()
                    .filter(RepoResponse::is_vault)
                    .map(RemoteRepo::from),
            );
            if last_page {
                break;
            }
        }
        Ok(found)
    }

    fn repo_access(&self, token: &str, full_name: &str) -> Result<RepoAccess, CoreError> {
        let response = self
            .client
            .get(format!("{API}/repos/{full_name}"))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(network)?;
        // GitHub answers 404 for a private repository the user cannot see.
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(RepoAccess::None);
        }
        check(response.status())?;
        let repo: RepoResponse = response.json().map_err(network)?;
        Ok(repo.access())
    }

    fn create_repo(
        &self,
        token: &str,
        name: &str,
        description: &str,
    ) -> Result<RemoteRepo, CoreError> {
        let response = self
            .client
            .post(format!("{API}/user/repos"))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .json(&json!({ "name": name, "description": description, "private": true, "auto_init": false }))
            .send()
            .map_err(network)?;
        check(response.status())?;
        let repo: RepoResponse = response.json().map_err(network)?;
        self.add_topic(token, &repo.full_name)?;
        Ok(repo.into())
    }
}

impl GithubHttp {
    fn add_topic(&self, token: &str, full_name: &str) -> Result<(), CoreError> {
        let response = self
            .client
            .put(format!("{API}/repos/{full_name}/topics"))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .json(&json!({ "names": [VAULT_TOPIC] }))
            .send()
            .map_err(network)?;
        check(response.status())
    }
}
