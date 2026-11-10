//! The one place where real adapters are wired into `Services`.

use std::{path::PathBuf, sync::Arc};

use gitvault_core::{
    application::{
        context::{Context, CoreConfig},
        session::SessionManager,
    },
    ports::{Clock, KeyValueStore},
    services::Services,
    CoreError,
};
use gitvault_crypto::KdfParams;

use crate::{
    git::GitCli,
    github::GithubHttp,
    storage::{FileKeyValueStore, FsVaultStorage, JsonlAuditSink},
    system::{GhCliProcess, SystemClock, UuidGenerator},
};

/// How the front door wants GitVault configured.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Directory for vaults, state and the audit log.
    pub home: PathBuf,
    /// Shown in the audit log, e.g. `CLI` or `Desktop`.
    pub device_label: String,
    pub github_client_id: Option<String>,
    /// Keep unlocked keys across processes (CLI) or in memory only (desktop).
    pub persist_sessions: bool,
    pub kdf: KdfParams,
}

impl AppConfig {
    /// Defaults plus the `GITVAULT_HOME` and `GITVAULT_GITHUB_CLIENT_ID` environment variables.
    #[must_use]
    pub fn from_env(device_label: &str, persist_sessions: bool) -> Self {
        let home = std::env::var_os("GITVAULT_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::data_dir().map(|dir| dir.join("gitvault")))
            .unwrap_or_else(|| PathBuf::from(".gitvault-data"));
        Self {
            home,
            device_label: device_label.to_owned(),
            github_client_id: std::env::var("GITVAULT_GITHUB_CLIENT_ID").ok(),
            persist_sessions,
            kdf: KdfParams::default(),
        }
    }
}

/// Builds the `Services` used by every front door (CLI, desktop app).
///
/// # Errors
/// Fails if the HTTP client cannot be created.
pub fn build_services(config: &AppConfig) -> Result<Services, CoreError> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let kv: Arc<dyn KeyValueStore> =
        Arc::new(FileKeyValueStore::new(config.home.join("state.json")));
    let vaults_root = config.home.join("vaults");
    let sessions =
        SessionManager::new(Arc::clone(&kv), Arc::clone(&clock), config.persist_sessions);
    let context = Context {
        clock,
        ids: Arc::new(UuidGenerator),
        kv,
        storage: Arc::new(FsVaultStorage::new(vaults_root.clone())),
        vcs: Arc::new(GitCli::new(vaults_root)),
        host: Arc::new(GithubHttp::new()?),
        gh_cli: Arc::new(GhCliProcess),
        audit: Arc::new(JsonlAuditSink::new(config.home.join("audit.jsonl"))),
        sessions,
        config: CoreConfig {
            device_name: format!("{} · {}", std::env::consts::OS, config.device_label),
            github_client_id: config.github_client_id.clone(),
            kdf: config.kdf.clone(),
        },
    };
    Ok(Services::new(context))
}
