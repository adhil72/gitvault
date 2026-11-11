//! Repository collaborators through GitHub's collaborator and invitation endpoints.

use gitvault_core::{domain::collaborator::CollaboratorRole, ports::Collaborator, CoreError};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::json;

use super::{
    http_host::{check, network, API},
    models::{CollaboratorResponse, InvitationResponse},
    GithubHttp,
};

const PER_PAGE: u32 = 100;
const MAX_PAGES: u32 = 5;

impl GithubHttp {
    /// Active collaborators first, then pending invitations.
    pub(super) fn fetch_collaborators(
        &self,
        token: &str,
        full_name: &str,
    ) -> Result<Vec<Collaborator>, CoreError> {
        let active: Vec<CollaboratorResponse> =
            self.get_all(&format!("{API}/repos/{full_name}/collaborators"), token)?;
        let mut found: Vec<Collaborator> = active.into_iter().map(Collaborator::from).collect();
        // Listing invitations needs admin rights; without them the active list is still useful.
        match self
            .get_all::<InvitationResponse>(&format!("{API}/repos/{full_name}/invitations"), token)
        {
            Ok(invitations) => {
                found.extend(
                    invitations
                        .into_iter()
                        .filter_map(InvitationResponse::into_collaborator),
                );
            }
            Err(CoreError::PermissionDenied(_)) => {}
            Err(other) => return Err(other),
        }
        Ok(found)
    }

    pub(super) fn put_collaborator(
        &self,
        token: &str,
        full_name: &str,
        login: &str,
        role: CollaboratorRole,
    ) -> Result<(), CoreError> {
        let permission = match role {
            CollaboratorRole::Read => "pull",
            CollaboratorRole::Write => "push",
        };
        let response = self
            .authorized(
                Method::PUT,
                &format!("{API}/repos/{full_name}/collaborators/{login}"),
                token,
            )
            .json(&json!({ "permission": permission }))
            .send()
            .map_err(network)?;
        check_member(response.status(), "GitHub user")
    }

    pub(super) fn delete_collaborator(
        &self,
        token: &str,
        full_name: &str,
        login: &str,
    ) -> Result<(), CoreError> {
        let response = self
            .authorized(
                Method::DELETE,
                &format!("{API}/repos/{full_name}/collaborators/{login}"),
                token,
            )
            .send()
            .map_err(network)?;
        check_member(response.status(), "collaborator")
    }

    fn authorized(
        &self,
        method: Method,
        url: &str,
        token: &str,
    ) -> reqwest::blocking::RequestBuilder {
        self.client
            .request(method, url)
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// Reads every page of a list endpoint, up to a fixed bound.
    fn get_all<T: DeserializeOwned>(&self, base: &str, token: &str) -> Result<Vec<T>, CoreError> {
        let mut all = Vec::new();
        for page in 1..=MAX_PAGES {
            let batch: Vec<T> =
                self.get(&format!("{base}?per_page={PER_PAGE}&page={page}"), token)?;
            let last_page = batch.len() < PER_PAGE as usize;
            all.extend(batch);
            if last_page {
                break;
            }
        }
        Ok(all)
    }
}

/// GitHub answers 404 when the user (or the collaborator) does not exist, and 422 when it
/// refuses an invitation; both deserve a message of their own, not a generic network error.
fn check_member(status: StatusCode, entity: &'static str) -> Result<(), CoreError> {
    match status {
        StatusCode::NOT_FOUND => Err(CoreError::NotFound { entity }),
        StatusCode::UNPROCESSABLE_ENTITY => Err(CoreError::Validation(
            "GitHub refused that invitation".into(),
        )),
        other => check(other),
    }
}
