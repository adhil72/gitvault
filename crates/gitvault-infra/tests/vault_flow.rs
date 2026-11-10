//! End-to-end tests of the core services with real files and real git.
#![allow(clippy::unwrap_used, clippy::too_many_lines)]

mod support;

use std::{fs, process::Command};

use gitvault_core::ports::GitHost;
use gitvault_core::{
    dto::{EnvFilter, ImportEnvInput, UnlockInput, UpdateSecretInput},
    ErrorCode,
};
use support::{add_secret_input, create_vault, device, list_all, FakeHost, PASSWORD};

#[test]
fn secret_lifecycle_with_history_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    let s = device(dir.path(), FakeHost::new(dir.path()), false);
    let vault = create_vault(&s, "Personal", false).vault.id.unwrap();

    let added = s
        .secrets
        .add(&add_secret_input(&vault, "STRIPE_KEY", "sk_live_1"))
        .unwrap();
    let id = added.summary.id.clone();
    assert_eq!(list_all(&s, &vault).len(), 1);
    assert_eq!(s.secrets.reveal(&vault, &id).unwrap().value, "sk_live_1");

    let updated = s
        .secrets
        .update(&UpdateSecretInput {
            vault_id: vault.clone(),
            secret_id: id.clone(),
            name: "STRIPE_KEY".into(),
            kind: added.summary.kind,
            value: Some("sk_live_2".into()),
            environment: Some("Production".into()),
            tags: vec![],
            notes: Some("rotated".into()),
            expires_at: None,
            remind_before_days: None,
            favorite: true,
        })
        .unwrap();
    assert_eq!(updated.summary.version, 2);
    assert!(updated.summary.favorite);

    let history = s.secrets.history(&vault, &id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].version, 1);

    s.secrets.restore(&vault, &id, &history[1].commit).unwrap();
    assert_eq!(s.secrets.reveal(&vault, &id).unwrap().value, "sk_live_1");
    assert_eq!(s.secrets.get(&vault, &id).unwrap().summary.version, 3);

    s.secrets.delete(&vault, &id).unwrap();
    assert!(list_all(&s, &vault).is_empty());
    let after_delete = s.secrets.history(&vault, &id).unwrap();
    s.secrets
        .restore(&vault, &id, &after_delete[1].commit)
        .unwrap();
    assert_eq!(list_all(&s, &vault).len(), 1);
}

#[test]
fn duplicate_names_are_rejected_per_environment() {
    let dir = tempfile::tempdir().unwrap();
    let s = device(dir.path(), FakeHost::new(dir.path()), false);
    let vault = create_vault(&s, "V", false).vault.id.unwrap();
    s.secrets
        .add(&add_secret_input(&vault, "KEY", "1"))
        .unwrap();
    let error = s
        .secrets
        .add(&add_secret_input(&vault, "key", "2"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::Conflict);
}

#[test]
fn locking_unlocking_and_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let s = device(dir.path(), FakeHost::new(dir.path()), false);
    let created = create_vault(&s, "V", false);
    let vault = created.vault.id.clone().unwrap();

    s.vaults.lock(&vault).unwrap();
    assert_eq!(
        s.secrets.facets(&vault).unwrap_err().code(),
        ErrorCode::VaultLocked
    );

    let wrong = UnlockInput {
        vault_id: vault.clone(),
        password: Some("nope nope nope".into()),
        recovery_key: None,
    };
    assert_eq!(
        s.vaults.unlock(&wrong).unwrap_err().code(),
        ErrorCode::WrongPassword
    );

    let recovery = UnlockInput {
        vault_id: vault.clone(),
        password: None,
        recovery_key: Some(created.recovery_key),
    };
    s.vaults.unlock(&recovery).unwrap();
    assert!(s.secrets.facets(&vault).is_ok());

    s.vaults
        .change_password(&vault, PASSWORD, "a brand new password")
        .unwrap();
    s.vaults.lock(&vault).unwrap();
    let new = UnlockInput {
        vault_id: vault,
        password: Some("a brand new password".into()),
        recovery_key: None,
    };
    assert!(s.vaults.unlock(&new).is_ok());
}

#[test]
fn sessions_persist_between_processes_only_when_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let host = FakeHost::new(dir.path());
    let first = device(dir.path(), host.clone(), true);
    let vault = create_vault(&first, "V", false).vault.id.unwrap();
    assert!(first.secrets.facets(&vault).is_ok());

    let second_cli = device(dir.path(), host.clone(), true);
    assert!(second_cli.secrets.facets(&vault).is_ok());
    let token = second_cli.vaults.export_session(&vault).unwrap();

    let desktop = device(dir.path(), host.clone(), false);
    assert_eq!(
        desktop.secrets.facets(&vault).unwrap_err().code(),
        ErrorCode::VaultLocked
    );
    assert_eq!(desktop.vaults.import_session(&token).unwrap(), vault);
    assert!(desktop.secrets.facets(&vault).is_ok());

    second_cli.vaults.lock(&vault).unwrap();
    assert_eq!(
        device(dir.path(), host, true)
            .secrets
            .facets(&vault)
            .unwrap_err()
            .code(),
        ErrorCode::VaultLocked
    );
}

#[test]
fn env_import_and_export_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let s = device(dir.path(), FakeHost::new(dir.path()), false);
    let vault = create_vault(&s, "V", false).vault.id.unwrap();
    let input = ImportEnvInput {
        vault_id: vault.clone(),
        content: "A=1\nB=\"two words\"\n# c\nC=3\n".into(),
        environment: Some("Development".into()),
        overwrite: false,
    };
    let report = s.secrets.import_env(&input).unwrap();
    assert_eq!((report.created, report.skipped), (3, 0));
    assert_eq!(s.secrets.import_env(&input).unwrap().skipped, 3);

    let filter = EnvFilter {
        environment: Some("development".into()),
        tag: None,
    };
    let exported = s.secrets.export_env(&vault, &filter).unwrap();
    assert_eq!(exported, "A=1\nB=\"two words\"\nC=3\n");
    let history = s.secrets.facets(&vault).unwrap();
    assert_eq!(history.total, 3);
}

#[test]
fn tampered_files_are_detected_and_nothing_sensitive_is_stored_in_plaintext() {
    let dir = tempfile::tempdir().unwrap();
    let s = device(dir.path(), FakeHost::new(dir.path()), false);
    let vault = create_vault(&s, "V", false).vault.id.unwrap();
    let id = s
        .secrets
        .add(&add_secret_input(
            &vault,
            "SUPER_SECRET_NAME",
            "hunter2-value",
        ))
        .unwrap()
        .summary
        .id;

    let repo = dir.path().join("vaults").join(&vault);
    let log = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["log", "-p", "--all"])
        .output()
        .unwrap();
    let everything = String::from_utf8_lossy(&log.stdout).into_owned();
    assert!(!everything.contains("SUPER_SECRET_NAME") && !everything.contains("hunter2-value"));
    assert!(!everything.contains("Stripe"));

    let file = repo.join("secrets").join(format!("{id}.enc"));
    let mut text = fs::read_to_string(&file).unwrap();
    let at = text.find("\"ciphertext\": \"").unwrap() + 15;
    let flipped = if &text[at..=at] == "A" { "B" } else { "A" };
    text.replace_range(at..=at, flipped);
    fs::write(&file, text).unwrap();
    assert!(s.secrets.reveal(&vault, &id).is_err());
}

#[test]
fn two_devices_sync_and_resolve_conflicts() {
    let dir = tempfile::tempdir().unwrap();
    let host = FakeHost::new(dir.path());
    let (home_a, home_b) = (dir.path().join("a"), dir.path().join("b"));
    let a = device(&home_a, host.clone(), false);
    let b = device(&home_b, host.clone(), false);
    a.auth.login_with_token("token-a").unwrap();
    b.auth.login_with_token("token-b").unwrap();

    let created = create_vault(&a, "Shared", true);
    assert!(created.remote_error.is_none(), "{:?}", created.remote_error);
    let vault = created.vault.id.unwrap();
    let secret = a
        .secrets
        .add(&add_secret_input(&vault, "API_KEY", "v0"))
        .unwrap()
        .summary
        .id;
    assert_eq!(a.sync.sync_now(&vault).unwrap().pushed, 1);

    let repo = host.find_vault_repos("t").unwrap().remove(0).full_name;
    b.vaults.connect(&repo).unwrap();
    b.vaults
        .unlock(&UnlockInput {
            vault_id: vault.clone(),
            password: Some(PASSWORD.into()),
            recovery_key: None,
        })
        .unwrap();
    assert_eq!(b.secrets.reveal(&vault, &secret).unwrap().value, "v0");

    // Both devices rotate the same secret.
    a.secrets.rotate(&vault, &secret, "from-a").unwrap();
    assert_eq!(a.sync.sync_now(&vault).unwrap().conflicts, 0);
    b.secrets.rotate(&vault, &secret, "from-b").unwrap();
    let result = b.sync.sync_now(&vault).unwrap();
    assert_eq!(result.conflicts, 1);
    assert_eq!(
        b.sync.conflicts(&vault).unwrap()[0].name.as_deref(),
        Some("API_KEY")
    );

    b.sync
        .resolve(&vault, &secret, &gitvault_core::dto::ConflictKeep::Remote)
        .unwrap();
    assert_eq!(b.secrets.reveal(&vault, &secret).unwrap().value, "from-a");
    assert_eq!(b.sync.sync_now(&vault).unwrap().conflicts, 0);

    // A changes something else; B receives it without conflict.
    a.secrets
        .add(&add_secret_input(&vault, "OTHER", "x"))
        .unwrap();
    a.sync.sync_now(&vault).unwrap();
    assert!(b.sync.sync_now(&vault).unwrap().pulled >= 1);
    assert_eq!(list_all(&b, &vault).len(), 2);
}
