use std::{collections::BTreeMap, fs};

use zeroize::Zeroizing;

use crate::{
    crypto::{KdfParams, RecoveryKey, SealedBlob, VaultKey},
    domain::{
        check_password, now_unix, open_env, seal_env, validate_slug, ProjectManifest, VaultHeader,
    },
    drivers::{
        paths::{ensure_private_dir, write_private_file},
        repo_name_setting, skip_gh_check, GhDriver, GitDriver, VaultPaths,
    },
    error::AppError,
    ui,
};

/// Controls how master passwords may be acquired during unlock/initialization.
#[derive(Debug, Clone, Copy)]
pub struct PasswordOptions {
    pub password_stdin: bool,
    pub non_interactive: bool,
}

#[derive(Debug, Clone)]
pub enum SyncResult {
    Synced,
    LocalOnly,
    CommittedOffline(String),
}

/// Manages the hidden encrypted repository and the local `0600` session key cache.
pub struct VaultStore {
    pub paths: VaultPaths,
}

impl VaultStore {
    pub fn open_default() -> Result<Self, AppError> {
        Ok(Self {
            paths: VaultPaths::discover()?,
        })
    }

    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.paths.header_file.is_file()
    }

    #[must_use]
    pub fn is_unlocked(&self) -> bool {
        self.paths.session_key.is_file()
    }

    /// Ensures local vault store exists (cloning/creating via `gh` when needed).
    /// Returns `Some((VaultKey, RecoveryKey))` ONLY on brand-new vault creation.
    pub fn ensure_initialized(
        &self,
        pw_opts: PasswordOptions,
    ) -> Result<Option<(VaultKey, RecoveryKey)>, AppError> {
        ensure_private_dir(&self.paths.home)?;
        if !self.is_initialized() {
            self.bootstrap_remote_repo()?;
        }
        if self.is_initialized() {
            return Ok(None);
        }

        GitDriver::ensure_initialized(&self.paths.store_dir)?;
        let password = read_new_password(pw_opts)?;
        let (header, key, recovery) =
            VaultHeader::create(&password, KdfParams::default(), now_unix())?;
        self.write_header(&header)?;
        self.cache_session_key(&key)?;

        if let SyncResult::CommittedOffline(reason) =
            self.commit_and_sync("vault: initialize encrypted store")?
        {
            ui::warn(&format!(
                "Vault initialized locally; push deferred ({reason}). Run 'gitvault sync' when online."
            ));
        }
        Ok(Some((key, recovery)))
    }

    /// Used by `load` / `unlock`: clones an existing remote vault if present, or fails cleanly.
    pub fn ensure_initialized_from_existing(&self) -> Result<(), AppError> {
        if self.is_initialized() {
            return Ok(());
        }
        if !skip_gh_check() {
            let (_, full_name) = resolve_remote_full_name()?;
            GhDriver::setup_git_auth()?;
            if GhDriver::repo_exists(&full_name)? {
                ui::info(&format!("Cloning vault repository '{full_name}'..."));
                GhDriver::clone_repo(&full_name, &self.paths.store_dir)?;
            }
        }
        if !self.is_initialized() {
            return Err(AppError::NotFound(
                "vault store not found on this machine or GitHub; run 'gitvault init' first".into(),
            ));
        }
        Ok(())
    }

    fn bootstrap_remote_repo(&self) -> Result<(), AppError> {
        if skip_gh_check() {
            return GitDriver::ensure_initialized(&self.paths.store_dir);
        }
        let (repo_name, full_name) = resolve_remote_full_name()?;
        GhDriver::setup_git_auth()?;

        if self.paths.store_dir.join(".git").exists() {
            let _ = GitDriver::pull_rebase(&self.paths.store_dir);
            return Ok(());
        }
        if GhDriver::repo_exists(&full_name)? {
            ui::info(&format!(
                "Cloning existing vault repository '{full_name}'..."
            ));
            GhDriver::clone_repo(&full_name, &self.paths.store_dir)?;
        } else {
            ui::info(&format!(
                "Creating private GitHub vault repository '{full_name}'..."
            ));
            GhDriver::create_private_repo(&repo_name)?;
            GhDriver::clone_repo(&full_name, &self.paths.store_dir)?;
        }
        GitDriver::ensure_initialized(&self.paths.store_dir)
    }

    pub fn read_header(&self) -> Result<VaultHeader, AppError> {
        if !self.is_initialized() {
            return Err(AppError::NotFound(
                "vault store is not initialized yet; run 'gitvault init'".into(),
            ));
        }
        let raw = fs::read_to_string(&self.paths.header_file)?;
        serde_json::from_str(&raw)
            .map_err(|e| AppError::Validation(format!("corrupt vault.json: {e}")))
    }

    pub fn write_header(&self, header: &VaultHeader) -> Result<(), AppError> {
        ensure_private_dir(&self.paths.store_dir)?;
        let mut text = serde_json::to_string_pretty(header)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        text.push('\n');
        fs::write(&self.paths.header_file, text)?;
        Ok(())
    }

    pub fn cache_session_key(&self, key: &VaultKey) -> Result<(), AppError> {
        let encoded = key.to_base64();
        write_private_file(&self.paths.session_key, encoded.as_bytes())
    }

    fn read_cached_key(&self, header: &VaultHeader) -> Option<VaultKey> {
        let raw = Zeroizing::new(fs::read_to_string(&self.paths.session_key).ok()?);
        let key = VaultKey::from_base64(&raw).ok()?;
        header.verify_key(&key).then_some(key)
    }

    /// Returns the unlocked [`VaultKey`] from session cache, `GITVAULT_PASSWORD`, or interactive prompt.
    pub fn unlock(
        &self,
        pw_opts: PasswordOptions,
        recovery_key: Option<&str>,
        force_prompt: bool,
    ) -> Result<VaultKey, AppError> {
        let header = self.read_header()?;
        if let Some(rec_text) = recovery_key {
            let key = header.unlock_with_recovery(&RecoveryKey::parse(rec_text)?)?;
            self.cache_session_key(&key)?;
            return Ok(key);
        }
        if !force_prompt {
            if let Some(cached) = self.read_cached_key(&header) {
                return Ok(cached);
            }
        }
        let password = read_unlock_password(pw_opts)?;
        let key = header.unlock_with_password(&password)?;
        self.cache_session_key(&key)?;
        Ok(key)
    }

    /// Overwrites and deletes the cached `session.key`.
    pub fn lock(&self) -> Result<bool, AppError> {
        if !self.paths.session_key.exists() {
            return Ok(false);
        }
        if let Ok(meta) = fs::metadata(&self.paths.session_key) {
            let zeros = vec![0_u8; meta.len().max(64) as usize];
            let _ = fs::write(&self.paths.session_key, zeros);
        }
        fs::remove_file(&self.paths.session_key)?;
        Ok(true)
    }

    pub fn pull_best_effort(&self) {
        if !self.paths.store_dir.join(".git").exists() {
            return;
        }
        if let Err(err) = GitDriver::pull_rebase(&self.paths.store_dir) {
            ui::warn(&format!(
                "Could not pull latest vault changes ({err}); using local copy."
            ));
        }
    }

    pub fn sync_strict(&self) -> Result<(bool, bool), AppError> {
        if !self.paths.store_dir.join(".git").exists() {
            return Err(AppError::NotFound(
                "vault store is not initialized yet; run 'gitvault init'".into(),
            ));
        }
        let _ = GitDriver::commit_all(&self.paths.store_dir, "vault: sync pending changes")?;
        let pulled = GitDriver::pull_rebase(&self.paths.store_dir)?;
        let pushed = GitDriver::push(&self.paths.store_dir)?;
        Ok((pulled, pushed))
    }

    pub fn commit_and_sync(&self, message: &str) -> Result<SyncResult, AppError> {
        GitDriver::commit_all(&self.paths.store_dir, message)?;
        if !GitDriver::has_remote(&self.paths.store_dir) {
            return Ok(SyncResult::LocalOnly);
        }
        match GitDriver::push(&self.paths.store_dir) {
            Ok(_) => Ok(SyncResult::Synced),
            Err(first_err) => {
                if GitDriver::pull_rebase(&self.paths.store_dir).is_ok()
                    && GitDriver::push(&self.paths.store_dir).is_ok()
                {
                    return Ok(SyncResult::Synced);
                }
                Ok(SyncResult::CommittedOffline(first_err.to_string()))
            }
        }
    }

    pub fn list_projects(&self) -> Result<Vec<String>, AppError> {
        if !self.paths.projects_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.paths.projects_dir)? {
            let path = entry?.path();
            if path.is_dir() && path.join("project.json").is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_owned());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn read_project(&self, project: &str) -> Result<ProjectManifest, AppError> {
        let project = validate_slug(project, "project")?;
        let path = self.paths.project_manifest(&project);
        if !path.is_file() {
            return Err(AppError::NotFound(format!(
                "project '{project}' does not exist in the vault"
            )));
        }
        let raw = fs::read_to_string(&path)?;
        serde_json::from_str(&raw)
            .map_err(|e| AppError::Validation(format!("corrupt project.json: {e}")))
    }

    pub fn write_project(&self, manifest: &ProjectManifest) -> Result<(), AppError> {
        ensure_private_dir(&self.paths.project_dir(&manifest.name))?;
        ensure_private_dir(&self.paths.envs_dir(&manifest.name))?;
        let mut text = serde_json::to_string_pretty(manifest)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        text.push('\n');
        fs::write(self.paths.project_manifest(&manifest.name), text)?;
        Ok(())
    }

    pub fn ensure_project(
        &self,
        project: &str,
        env: Option<&str>,
    ) -> Result<(ProjectManifest, bool), AppError> {
        let project = validate_slug(project, "project")?;
        let (mut manifest, created) = if self.paths.project_manifest(&project).is_file() {
            (self.read_project(&project)?, false)
        } else {
            (ProjectManifest::new(&project, now_unix())?, true)
        };
        let env_added = match env {
            Some(e) => manifest.ensure_environment(e)?,
            None => false,
        };
        if created || env_added {
            self.write_project(&manifest)?;
        }
        Ok((manifest, created))
    }

    pub fn ensure_environment(&self, project: &str, env: &str) -> Result<bool, AppError> {
        let mut manifest = self.read_project(project)?;
        let added = manifest.ensure_environment(env)?;
        if added {
            self.write_project(&manifest)?;
        }
        Ok(added)
    }

    pub fn delete_environment(&self, project: &str, env: &str) -> Result<(), AppError> {
        let project = validate_slug(project, "project")?;
        let env = validate_slug(env, "environment")?;
        let mut manifest = self.read_project(&project)?;
        if !manifest.remove_environment(&env) {
            return Err(AppError::NotFound(format!(
                "environment '{env}' not found in project '{project}'"
            )));
        }
        self.write_project(&manifest)?;
        let enc_path = self.paths.env_file(&project, &env);
        if enc_path.is_file() {
            fs::remove_file(enc_path)?;
        }
        Ok(())
    }

    pub fn read_env_vars(
        &self,
        key: &VaultKey,
        project: &str,
        env: &str,
    ) -> Result<BTreeMap<String, String>, AppError> {
        let project = validate_slug(project, "project")?;
        let env = validate_slug(env, "environment")?;
        let _ = self.read_project(&project)?;
        let path = self.paths.env_file(&project, &env);
        if !path.is_file() {
            return Ok(BTreeMap::new());
        }
        let raw = fs::read_to_string(&path)?;
        let blob: SealedBlob = serde_json::from_str(&raw)
            .map_err(|e| AppError::Crypto(format!("corrupt encrypted env file: {e}")))?;
        open_env(key, &project, &env, &blob)
    }

    pub fn write_env_vars(
        &self,
        key: &VaultKey,
        project: &str,
        env: &str,
        vars: &BTreeMap<String, String>,
    ) -> Result<(), AppError> {
        let project = validate_slug(project, "project")?;
        let env = validate_slug(env, "environment")?;
        self.ensure_environment(&project, &env)?;
        let blob = seal_env(key, &project, &env, vars)?;
        let mut text =
            serde_json::to_string_pretty(&blob).map_err(|e| AppError::Crypto(e.to_string()))?;
        text.push('\n');
        let env_path = self.paths.env_file(&project, &env);
        if let Some(parent) = env_path.parent() {
            ensure_private_dir(parent)?;
        }
        fs::write(env_path, text)?;
        Ok(())
    }
}

fn read_new_password(opts: PasswordOptions) -> Result<Zeroizing<String>, AppError> {
    if opts.password_stdin {
        let pw = ui::read_stdin_secret()?;
        check_password(&pw)?;
        return Ok(pw);
    }
    if let Ok(env_pw) = std::env::var("GITVAULT_PASSWORD") {
        if !env_pw.is_empty() {
            let pw = Zeroizing::new(env_pw);
            check_password(&pw)?;
            return Ok(pw);
        }
    }
    if opts.non_interactive {
        return Err(AppError::VaultLocked {
            hint: "Pass --password-stdin or set GITVAULT_PASSWORD to initialize non-interactively."
                .into(),
        });
    }
    prompt_confirmed_password("First-time setup: choose a master password to encrypt your vault.")
}

pub fn prompt_confirmed_password(banner: &str) -> Result<Zeroizing<String>, AppError> {
    ui::info(banner);
    loop {
        let first = ui::password("Master password (min 8 chars)")?;
        if let Err(err) = check_password(&first) {
            ui::warn(&err.to_string());
            continue;
        }
        let second = ui::password("Confirm master password")?;
        if first.as_str() != second.as_str() {
            ui::warn("Passwords do not match. Try again.");
            continue;
        }
        return Ok(first);
    }
}

fn read_unlock_password(opts: PasswordOptions) -> Result<Zeroizing<String>, AppError> {
    if opts.password_stdin {
        return ui::read_stdin_secret();
    }
    if let Ok(env_pw) = std::env::var("GITVAULT_PASSWORD") {
        if !env_pw.is_empty() {
            return Ok(Zeroizing::new(env_pw));
        }
    }
    if opts.non_interactive {
        return Err(AppError::VaultLocked {
            hint: "Run 'gitvault unlock' or set GITVAULT_PASSWORD.".into(),
        });
    }
    ui::password("Master password")
}

fn resolve_remote_full_name() -> Result<(String, String), AppError> {
    let repo_setting = repo_name_setting();
    if let Some((_, repo)) = repo_setting.split_once('/') {
        return Ok((repo.to_owned(), repo_setting));
    }
    let user = GhDriver::current_user()?;
    let full_name = format!("{user}/{repo_setting}");
    Ok((repo_setting, full_name))
}
