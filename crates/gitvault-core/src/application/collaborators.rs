//! Collaborators: GitHub users who get access to a vault's repository.
//!
//! GitHub's repository permissions are the access list (see `require_repo_access`), so
//! inviting someone grants repository access only. Opening the vault still needs the master
//! password or a trusted-device key; sharing those is a separate, deliberate step.

use super::vaults::{parse_repo_url, VaultService};
use crate::{
    domain::{audit::AuditKind, collaborator::GithubLogin, ids::VaultId},
    dto::{CollaboratorDto, InviteCollaboratorInput},
    error::CoreError,
    ports::Collaborator,
};

impl VaultService {
    /// Active collaborators and pending invitations of the vault's repository.
    ///
    /// # Errors
    /// [`CoreError::Validation`] for a local-only vault, [`CoreError::NotAuthenticated`]
    /// when signed out, and GitHub errors (permission, network).
    pub fn list_collaborators(&self, vault_id: &str) -> Result<Vec<CollaboratorDto>, CoreError> {
        let (token, repo) = self.collaboration_target(vault_id)?;
        let found = self.ctx.host.list_collaborators(&token, &repo)?;
        Ok(found.into_iter().map(to_dto).collect())
    }

    /// Invites a GitHub user to the vault's repository, or changes their role.
    ///
    /// # Errors
    /// [`CoreError::Validation`] for a bad username or a local-only vault,
    /// [`CoreError::NotFound`] for an unknown user, and GitHub errors.
    pub fn invite_collaborator(&self, input: &InviteCollaboratorInput) -> Result<(), CoreError> {
        let login = GithubLogin::parse(&input.login)?;
        let (token, repo) = self.collaboration_target(&input.vault_id)?;
        self.ctx
            .host
            .invite_collaborator(&token, &repo, login.as_str(), input.role)?;
        self.record_collaboration(&input.vault_id, AuditKind::CollaboratorInvited, &login);
        Ok(())
    }

    /// Removes a collaborator, or cancels their pending invitation.
    ///
    /// # Errors
    /// [`CoreError::Validation`] for a bad username or a local-only vault, and GitHub errors.
    pub fn remove_collaborator(&self, vault_id: &str, login: &str) -> Result<(), CoreError> {
        let login = GithubLogin::parse(login)?;
        let (token, repo) = self.collaboration_target(vault_id)?;
        self.ctx
            .host
            .remove_collaborator(&token, &repo, login.as_str())?;
        self.record_collaboration(vault_id, AuditKind::CollaboratorRemoved, &login);
        Ok(())
    }

    /// The GitHub token and `owner/name` of the vault's repository.
    fn collaboration_target(&self, vault_id: &str) -> Result<(String, String), CoreError> {
        let id = VaultId::parse(vault_id)?;
        let url = self.ctx.vcs.remote_url(&id)?.ok_or_else(|| {
            CoreError::Validation("this vault has no GitHub repository to share".into())
        })?;
        let repo = parse_repo_url(&url).ok_or_else(|| {
            CoreError::Validation("the vault's remote is not a GitHub repository".into())
        })?;
        Ok((self.ctx.token()?, repo))
    }

    fn record_collaboration(&self, vault_id: &str, kind: AuditKind, login: &GithubLogin) {
        if let Ok(id) = VaultId::parse(vault_id) {
            self.ctx.record(&id, kind, None, Some(login.as_str()));
        }
    }
}

fn to_dto(collaborator: Collaborator) -> CollaboratorDto {
    CollaboratorDto {
        login: collaborator.login,
        avatar_url: collaborator.avatar_url,
        role: collaborator.role,
        pending: collaborator.pending,
    }
}
