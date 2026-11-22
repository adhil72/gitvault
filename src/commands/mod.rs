pub mod auth_key;
pub mod config;
pub mod config_interactive;
pub mod doctor;
pub mod init;
pub mod load;
pub mod sync;

use std::{collections::BTreeMap, path::PathBuf};

use zeroize::Zeroize;

use crate::{
    crypto::VaultKey,
    drivers::{PasswordOptions, Prerequisites, SyncResult, VaultStore},
    error::AppError,
    ui,
};

pub use auth_key::{lock, unlock};
pub use config::config;
pub use doctor::doctor;
pub use init::init;
pub use load::{load, shell_init, unload};
pub use sync::{projects, sync};

/// Holds decrypted secret variables and scrubs every value from RAM on drop.
pub struct SecretVars(pub BTreeMap<String, String>);

impl Drop for SecretVars {
    fn drop(&mut self) {
        for value in self.0.values_mut() {
            value.zeroize();
        }
    }
}

pub struct EnvTarget<'a> {
    pub store: &'a VaultStore,
    pub key: &'a VaultKey,
    pub project: &'a str,
    pub env: &'a str,
}

pub fn current_dir() -> Result<PathBuf, AppError> {
    Ok(std::env::current_dir()?)
}

pub fn print_recovery_banner(recovery_display: &str) {
    ui::warn("Save this recovery key in a safe place (shown once):");
    eprintln!("  {}\n", ui::bold(recovery_display));
}

pub fn ensure_store_and_unlock(
    store: &VaultStore,
    pw_opts: PasswordOptions,
    pull: bool,
) -> Result<VaultKey, AppError> {
    Prerequisites::ensure_ready()?;
    if let Some((key, recovery)) = store.ensure_initialized(pw_opts)? {
        print_recovery_banner(&recovery.to_display());
        return Ok(key);
    }
    if pull {
        store.pull_best_effort();
    }
    store.unlock(pw_opts, None, false)
}

pub fn print_sync_outcome(result: SyncResult, action: &str) {
    match result {
        SyncResult::Synced => ui::success(&format!("{action} and synced to GitHub")),
        SyncResult::LocalOnly => ui::success(&format!("{action} (saved locally)")),
        SyncResult::CommittedOffline(reason) => {
            ui::warn(&format!("{action} locally; sync deferred ({reason})"));
        }
    }
}
