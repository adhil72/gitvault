use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::error::AppError;

const DEFAULT_REPO_NAME: &str = "gitvault-store";

#[derive(Debug, Clone)]
pub struct VaultPaths {
    pub home: PathBuf,
    pub store_dir: PathBuf,
    pub session_key: PathBuf,
    pub header_file: PathBuf,
    pub projects_dir: PathBuf,
}

impl VaultPaths {
    pub fn discover() -> Result<Self, AppError> {
        let home = resolve_home_dir()?;
        Ok(Self::from_home(home))
    }

    #[must_use]
    pub fn from_home(home: PathBuf) -> Self {
        let store_dir = home.join("store");
        let session_key = home.join("session.key");
        let header_file = store_dir.join("vault.json");
        let projects_dir = store_dir.join("projects");
        Self {
            home,
            store_dir,
            session_key,
            header_file,
            projects_dir,
        }
    }

    #[must_use]
    pub fn project_dir(&self, project: &str) -> PathBuf {
        self.projects_dir.join(project)
    }

    #[must_use]
    pub fn project_manifest(&self, project: &str) -> PathBuf {
        self.project_dir(project).join("project.json")
    }

    #[must_use]
    pub fn envs_dir(&self, project: &str) -> PathBuf {
        self.project_dir(project).join("envs")
    }

    #[must_use]
    pub fn env_file(&self, project: &str, env: &str) -> PathBuf {
        self.envs_dir(project).join(format!("{env}.enc"))
    }
}

fn non_empty_env(name: &str) -> Option<PathBuf> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

fn resolve_home_dir() -> Result<PathBuf, AppError> {
    if let Some(custom) = non_empty_env("GITVAULT_HOME") {
        return Ok(custom);
    }
    if let Some(xdg) = non_empty_env("XDG_DATA_HOME") {
        return Ok(xdg.join("gitvault"));
    }
    if let Some(home) = non_empty_env("HOME") {
        return Ok(home.join(".local/share/gitvault"));
    }
    Err(AppError::NotFound(
        "cannot determine data directory (HOME / GITVAULT_HOME unset)".into(),
    ))
}

#[must_use]
pub fn repo_name_setting() -> String {
    std::env::var("GITVAULT_REPO")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_REPO_NAME.to_owned())
}

#[must_use]
pub fn skip_gh_check() -> bool {
    std::env::var("GITVAULT_SKIP_GH_CHECK")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub fn ensure_private_dir(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    let mut file = fs::File::create(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
