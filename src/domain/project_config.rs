use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{domain::validate_slug, error::AppError};

pub const CONFIG_FILE_NAME: &str = "gitvault.json";

/// Non-secret project binding saved as `gitvault.json` inside a project repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: String,
    pub environment: String,
}

impl ProjectConfig {
    pub fn new(project: &str, environment: &str) -> Result<Self, AppError> {
        Ok(Self {
            project: validate_slug(project, "project")?,
            environment: validate_slug(environment, "environment")?,
        })
    }

    /// Searches `start` and its parent directories for `gitvault.json`.
    pub fn find_in_ancestors(start: &Path) -> Result<Option<(PathBuf, Self)>, AppError> {
        let mut dir = Some(start);
        while let Some(current) = dir {
            let candidate = current.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                let cfg = read_config_file(&candidate)?;
                return Ok(Some((candidate, cfg)));
            }
            dir = current.parent();
        }
        Ok(None)
    }

    /// Writes `gitvault.json` cleanly into `dir`.
    pub fn save_to_dir(&self, dir: &Path) -> Result<PathBuf, AppError> {
        let validated = Self::new(&self.project, &self.environment)?;
        let path = dir.join(CONFIG_FILE_NAME);
        let mut text = serde_json::to_string_pretty(&validated)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        text.push('\n');
        fs::write(&path, text)?;
        Ok(path)
    }

    /// Resolves `(project, environment)` from explicit CLI flags, falling back to `gitvault.json`.
    pub fn resolve(
        project_flag: Option<&str>,
        env_flag: Option<&str>,
        cwd: &Path,
    ) -> Result<(String, String), AppError> {
        if let (Some(p), Some(e)) = (project_flag, env_flag) {
            return Ok((
                validate_slug(p, "project")?,
                validate_slug(e, "environment")?,
            ));
        }
        let from_file = Self::find_in_ancestors(cwd)?.map(|(_, cfg)| cfg);
        let project = pick_project(project_flag, from_file.as_ref())?;
        let env = pick_env(env_flag, from_file.as_ref())?;
        Ok((project, env))
    }
}

fn read_config_file(candidate: &Path) -> Result<ProjectConfig, AppError> {
    let raw = fs::read_to_string(candidate)?;
    let parsed: ProjectConfig = serde_json::from_str(&raw).map_err(|e| {
        AppError::Validation(format!(
            "invalid {} at {}: {e}",
            CONFIG_FILE_NAME,
            candidate.display()
        ))
    })?;
    ProjectConfig::new(&parsed.project, &parsed.environment)
}

fn pick_project(flag: Option<&str>, file_cfg: Option<&ProjectConfig>) -> Result<String, AppError> {
    match (flag, file_cfg) {
        (Some(p), _) => validate_slug(p, "project"),
        (None, Some(cfg)) => Ok(cfg.project.clone()),
        (None, None) => Err(AppError::NotFound(
            "no gitvault.json found here (run 'gitvault init' or pass -p <project> -e <env>)"
                .into(),
        )),
    }
}

fn pick_env(flag: Option<&str>, file_cfg: Option<&ProjectConfig>) -> Result<String, AppError> {
    match (flag, file_cfg) {
        (Some(e), _) => validate_slug(e, "environment"),
        (None, Some(cfg)) => Ok(cfg.environment.clone()),
        (None, None) => Err(AppError::NotFound(
            "no environment specified (pass -e <env> or run 'gitvault init')".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolves_config_from_ancestors_and_cli_overrides() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("services/api");
        fs::create_dir_all(&nested).unwrap();

        let cfg = ProjectConfig::new("shop-api", "development").unwrap();
        cfg.save_to_dir(dir.path()).unwrap();

        assert_eq!(
            ProjectConfig::resolve(None, None, &nested).unwrap(),
            ("shop-api".into(), "development".into())
        );
        assert_eq!(
            ProjectConfig::resolve(None, Some("production"), &nested).unwrap(),
            ("shop-api".into(), "production".into())
        );
        assert_eq!(
            ProjectConfig::resolve(Some("other"), Some("staging"), &nested).unwrap(),
            ("other".into(), "staging".into())
        );
    }
}
