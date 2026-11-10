//! Shared helpers for integration tests: a fake GitHub backed by local bare repositories.
#![allow(clippy::unwrap_used, dead_code)]

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

use gitvault_core::{
    application::{
        context::{Context, CoreConfig},
        session::SessionManager,
    },
    domain::account::GithubAccount,
    dto::{CreateVaultInput, CreatedVault},
    ports::{Clock, DeviceCode, DevicePoll, GitHost, KeyValueStore, RemoteRepo, RepoAccess},
    services::Services,
    CoreError,
};
use gitvault_crypto::KdfParams;
use gitvault_infra::{
    git::GitCli,
    storage::{FileKeyValueStore, FsVaultStorage, JsonlAuditSink},
    system::{GhCliProcess, SystemClock, UuidGenerator},
};

pub const PASSWORD: &str = "correct horse battery";

/// GitHub stand-in: "repositories" are bare git repos in a temp directory.
pub struct FakeHost {
    base: PathBuf,
    repos: Mutex<Vec<RemoteRepo>>,
}

impl FakeHost {
    pub fn new(base: &Path) -> Arc<Self> {
        Arc::new(Self {
            base: base.to_path_buf(),
            repos: Mutex::new(Vec::new()),
        })
    }
}

impl GitHost for FakeHost {
    fn device_code(&self, _: &str) -> Result<DeviceCode, CoreError> {
        Err(CoreError::Network("not available in tests".into()))
    }

    fn poll_device_token(&self, _: &str, _: &str) -> Result<DevicePoll, CoreError> {
        Ok(DevicePoll::Pending)
    }

    fn current_user(&self, _: &str) -> Result<GithubAccount, CoreError> {
        Ok(GithubAccount {
            login: "octocat".into(),
            id: 1,
            name: None,
            avatar_url: String::new(),
        })
    }

    fn find_vault_repos(&self, _: &str) -> Result<Vec<RemoteRepo>, CoreError> {
        Ok(self.repos.lock().unwrap().clone())
    }

    fn repo_access(&self, _: &str, _: &str) -> Result<RepoAccess, CoreError> {
        Ok(RepoAccess::Write)
    }

    fn create_repo(&self, _: &str, name: &str, _: &str) -> Result<RemoteRepo, CoreError> {
        let path = self.base.join(format!("{name}.git"));
        let status = Command::new("git")
            .args(["init", "--bare", "-b", "main"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(status.status.success());
        let repo = RemoteRepo {
            full_name: format!("octocat/{name}"),
            clone_url: path.to_string_lossy().into_owned(),
            private: true,
            updated_at: String::new(),
        };
        self.repos.lock().unwrap().push(repo.clone());
        Ok(repo)
    }
}

/// One "device": its own home directory, state and vault clones.
pub fn device(home: &Path, host: Arc<dyn GitHost>, persist_sessions: bool) -> Services {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let kv: Arc<dyn KeyValueStore> = Arc::new(FileKeyValueStore::new(home.join("state.json")));
    let root = home.join("vaults");
    Services::new(Context {
        clock: Arc::clone(&clock),
        ids: Arc::new(UuidGenerator),
        kv: Arc::clone(&kv),
        storage: Arc::new(FsVaultStorage::new(root.clone())),
        vcs: Arc::new(GitCli::new(root)),
        host,
        gh_cli: Arc::new(GhCliProcess),
        audit: Arc::new(JsonlAuditSink::new(home.join("audit.jsonl"))),
        sessions: SessionManager::new(kv, clock, persist_sessions),
        config: CoreConfig {
            device_name: "test".into(),
            github_client_id: Some("client".into()),
            kdf: KdfParams::for_tests(),
        },
    })
}

pub fn create_vault(services: &Services, name: &str, remote: bool) -> CreatedVault {
    services
        .vaults
        .create(&CreateVaultInput {
            name: name.into(),
            description: None,
            repo_name: None,
            create_remote: remote,
            master_password: PASSWORD.into(),
        })
        .unwrap()
}

pub fn add_secret_input(
    vault_id: &str,
    name: &str,
    value: &str,
) -> gitvault_core::dto::AddSecretInput {
    gitvault_core::dto::AddSecretInput {
        vault_id: vault_id.into(),
        name: name.into(),
        kind: gitvault_core::domain::secret::SecretKind::ApiKey,
        value: value.into(),
        environment: Some("Production".into()),
        tags: vec!["billing".into()],
        notes: None,
        expires_at: None,
        remind_before_days: None,
    }
}

pub fn list_all(services: &Services, vault_id: &str) -> Vec<gitvault_core::dto::SecretSummary> {
    services
        .secrets
        .list(&gitvault_core::dto::ListSecretsQuery {
            vault_id: vault_id.into(),
            search: None,
            environment: None,
            tag: None,
            kind: None,
            favorites_only: false,
            expiring_within_days: None,
            sort: None,
        })
        .unwrap()
}
