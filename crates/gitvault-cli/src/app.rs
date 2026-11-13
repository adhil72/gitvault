//! Shared state of one CLI invocation.

use std::{io::IsTerminal, path::PathBuf};

use anyhow::{bail, Result};
use gitvault_core::{
    domain::vault::TrustContext,
    dto::{ListSecretsQuery, SecretSummary, UnlockInput},
    services::Services,
    CoreError,
};
use serde::Serialize;

const PASSWORD_ENV: &str = "GITVAULT_PASSWORD";
const DEVICE_KEY_ENV: &str = "GITVAULT_DEVICE_KEY";

pub struct App {
    pub services: Services,
    pub json: bool,
    pub interactive: bool,
    vault_flag: Option<String>,
}

impl App {
    pub fn new(
        services: Services,
        json: bool,
        non_interactive: bool,
        vault: Option<String>,
    ) -> Self {
        let interactive = !non_interactive && std::io::stdin().is_terminal();
        Self {
            services,
            json,
            interactive,
            vault_flag: vault,
        }
    }

    /// The vault named by `--vault` / `GITVAULT_VAULT`, if any (menus skip asking then).
    pub fn vault_flag(&self) -> Option<&str> {
        self.vault_flag.as_deref()
    }

    /// The vault to operate on: `--vault`, `GITVAULT_VAULT`, or the current default.
    pub fn vault_id(&self) -> Result<String> {
        if let Some(id) = &self.vault_flag {
            return Ok(id.clone());
        }
        match self.services.vaults.current() {
            Some(id) => Ok(id),
            None => Err(CoreError::Validation(
                "no vault selected: use --vault <id> or `gitvault vault use <id>`".into(),
            )
            .into()),
        }
    }

    /// The current directory with symlinks and `..` resolved, as folder trust compares it.
    pub fn current_folder() -> Option<PathBuf> {
        std::env::current_dir().ok()?.canonicalize().ok()
    }

    /// Silent unlock by a trusted device, judged from the current folder and environment.
    pub fn unlock_trusted_here(
        &self,
        vault_id: &str,
        device_key: Option<&str>,
        environment: Option<&str>,
    ) -> Result<(), CoreError> {
        let folder = Self::current_folder();
        let context = TrustContext {
            folder: folder.as_deref(),
            environment,
        };
        self.services
            .vaults
            .unlock_trusted(vault_id, device_key, &context)
            .map(|_| ())
    }

    /// Makes sure the vault is unlocked without ever prompting: a trusted device unlocks by
    /// itself, servers and CI set `GITVAULT_DEVICE_KEY` or `GITVAULT_PASSWORD`. The variable is removed from this process so child processes
    /// (`gitvault run`) never inherit it.
    pub fn ensure_unlocked(&self, environment: Option<&str>) -> Result<String> {
        let vault_id = self.vault_id()?;
        let password = std::env::var(PASSWORD_ENV).ok();
        let device_key = std::env::var(DEVICE_KEY_ENV).ok();
        std::env::remove_var(PASSWORD_ENV);
        std::env::remove_var(DEVICE_KEY_ENV);
        if self.services.vaults.info(&vault_id)?.unlocked {
            return Ok(vault_id);
        }
        // Order: an explicit device key (servers), this device's own key, then the password.
        match self.unlock_trusted_here(&vault_id, device_key.as_deref(), environment) {
            Ok(()) => return Ok(vault_id),
            Err(CoreError::VaultLocked) => {}
            Err(other) => return Err(other.into()),
        }
        let Some(password) = password else {
            return Err(CoreError::VaultLocked.into());
        };
        self.services.vaults.unlock(&UnlockInput {
            vault_id: vault_id.clone(),
            password: Some(password),
            recovery_key: None,
        })?;
        Ok(vault_id)
    }

    /// Prints JSON (machine mode) or the human text.
    pub fn emit<T: Serialize>(&self, value: &T, human: impl FnOnce() -> String) -> Result<()> {
        if self.json {
            println!("{}", serde_json::to_string_pretty(value)?);
        } else {
            let text = human();
            if !text.is_empty() {
                println!("{text}");
            }
        }
        Ok(())
    }

    /// A progress or hint message. Always stderr so stdout stays machine-readable.
    pub fn note(&self, message: &str) {
        eprintln!("{message}");
    }

    /// Finds a secret by id, or by name (optionally within one environment).
    pub fn resolve_secret(
        &self,
        identifier: &str,
        environment: Option<&str>,
    ) -> Result<SecretSummary> {
        let vault = self.vault_id()?;
        let all = self.services.secrets.list(&ListSecretsQuery {
            vault_id: vault,
            search: None,
            environment: environment.map(str::to_owned),
            tag: None,
            kind: None,
            favorites_only: false,
            expiring_within_days: None,
            sort: None,
        })?;
        if let Some(found) = all.iter().find(|s| s.id == identifier) {
            return Ok(found.clone());
        }
        let mut by_name: Vec<SecretSummary> = all
            .into_iter()
            .filter(|s| s.name.eq_ignore_ascii_case(identifier))
            .collect();
        match by_name.len() {
            0 => Err(CoreError::NotFound { entity: "secret" }.into()),
            1 => Ok(by_name.remove(0)),
            _ => bail!(CoreError::Validation(format!(
                "several secrets are named {identifier}: pass --environment or use the secret id"
            ))),
        }
    }
}
