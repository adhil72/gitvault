//! Inviting, listing and removing collaborators of a vault repository.
#![allow(clippy::unwrap_used)]

mod support;

use gitvault_core::{
    domain::{audit::AuditKind, collaborator::CollaboratorRole, ids::VaultId},
    dto::{AuditQuery, InviteCollaboratorInput},
    services::Services,
    ErrorCode,
};
use support::{create_vault, device, FakeHost};

/// A device signed in to GitHub (the fake host accepts any token).
fn signed_in_device(dir: &std::path::Path) -> gitvault_core::services::Services {
    let s = device(dir, FakeHost::new(dir), false);
    s.auth.login_with_token("test-token").unwrap();
    s
}

/// A vault whose remote is a GitHub repository. The URL is only recorded, never pushed to,
/// because the fake host answers for any repository.
fn vault_on_github(s: &Services) -> String {
    let vault = create_vault(s, "Team", false).vault.id.unwrap();
    let id = VaultId::parse(&vault).unwrap();
    s.ctx
        .vcs
        .set_remote(&id, "https://github.com/octocat/team.git")
        .unwrap();
    vault
}

fn invite(vault_id: &str, login: &str, role: CollaboratorRole) -> InviteCollaboratorInput {
    InviteCollaboratorInput {
        vault_id: vault_id.into(),
        login: login.into(),
        role,
    }
}

#[test]
fn invite_list_and_remove_a_collaborator() {
    let dir = tempfile::tempdir().unwrap();
    let s = signed_in_device(dir.path());
    let vault = vault_on_github(&s);

    assert!(s.vaults.list_collaborators(&vault).unwrap().is_empty());

    s.vaults
        .invite_collaborator(&invite(&vault, "@alice", CollaboratorRole::Write))
        .unwrap();
    let listed = s.vaults.list_collaborators(&vault).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].login, "alice");
    assert_eq!(listed[0].role, CollaboratorRole::Write);
    assert!(listed[0].pending);

    s.vaults.remove_collaborator(&vault, "alice").unwrap();
    assert!(s.vaults.list_collaborators(&vault).unwrap().is_empty());
}

#[test]
fn invitations_and_removals_are_audited_without_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let s = signed_in_device(dir.path());
    let vault = vault_on_github(&s);

    s.vaults
        .invite_collaborator(&invite(&vault, "bob", CollaboratorRole::Read))
        .unwrap();
    s.vaults.remove_collaborator(&vault, "bob").unwrap();

    let query = AuditQuery {
        vault_id: None,
        kind: None,
        search: None,
        limit: None,
    };
    let events = s.audit.list(&query).unwrap();
    let kinds: Vec<AuditKind> = events.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&AuditKind::CollaboratorInvited));
    assert!(kinds.contains(&AuditKind::CollaboratorRemoved));
}

#[test]
fn a_bad_username_is_rejected_before_reaching_github() {
    let dir = tempfile::tempdir().unwrap();
    let s = signed_in_device(dir.path());
    let vault = vault_on_github(&s);

    let error = s
        .vaults
        .invite_collaborator(&invite(&vault, "not a user", CollaboratorRole::Read))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::Validation);
}

#[test]
fn local_only_vaults_have_nobody_to_share_with() {
    let dir = tempfile::tempdir().unwrap();
    let s = signed_in_device(dir.path());
    let vault = create_vault(&s, "Solo", false).vault.id.unwrap();

    let error = s.vaults.list_collaborators(&vault).unwrap_err();
    assert_eq!(error.code(), ErrorCode::Validation);
}

#[test]
fn removing_someone_who_has_no_access_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let s = signed_in_device(dir.path());
    let vault = vault_on_github(&s);

    let error = s.vaults.remove_collaborator(&vault, "carol").unwrap_err();
    assert_eq!(error.code(), ErrorCode::NotFound);
}
