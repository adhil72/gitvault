use std::path::Path;

use crate::{
    cli::InitArgs,
    commands::{current_dir, ensure_store_and_unlock, print_sync_outcome},
    domain::{validate_slug, ProjectConfig},
    drivers::{GitDriver, PasswordOptions, VaultStore},
    error::AppError,
    ui,
};

pub fn init(args: InitArgs, pw_opts: PasswordOptions) -> Result<(), AppError> {
    let cwd = current_dir()?;
    let store = VaultStore::open_default()?;
    let _key = ensure_store_and_unlock(&store, pw_opts, !args.no_sync)?;

    let project = resolve_init_project(&store, args.project.as_deref(), pw_opts, &cwd)?;
    let (manifest, created_project) = store.ensure_project(&project, None)?;
    let environment =
        resolve_init_env(&manifest.environments, args.environment.as_deref(), pw_opts)?;
    let env_added = store.ensure_environment(&project, &environment)?;

    ProjectConfig::new(&project, &environment)?.save_to_dir(&cwd)?;
    sync_init_changes(
        &store,
        &project,
        &environment,
        created_project || env_added,
        args.no_sync,
    )?;

    ui::success("Created gitvault.json (zero .env files)");
    if cwd.join(".env").is_file() {
        ui::info("Tip: legacy .env detected — run 'gitvault config import .env --delete-source' to migrate and remove it.");
    }
    Ok(())
}

fn sync_init_changes(
    store: &VaultStore,
    project: &str,
    environment: &str,
    changed: bool,
    no_sync: bool,
) -> Result<(), AppError> {
    if !changed {
        ui::success(&format!("Linked '{project}' ({environment})"));
        return Ok(());
    }
    let msg = format!("project: initialize {project}/{environment}");
    if no_sync {
        let _ = GitDriver::commit_all(&store.paths.store_dir, &msg)?;
        ui::success(&format!("Initialized '{project}' ({environment})"));
    } else {
        let res = store.commit_and_sync(&msg)?;
        print_sync_outcome(res, &format!("Initialized '{project}' ({environment})"));
    }
    Ok(())
}

fn resolve_init_project(
    store: &VaultStore,
    flag: Option<&str>,
    pw_opts: PasswordOptions,
    cwd: &Path,
) -> Result<String, AppError> {
    if let Some(p) = flag {
        return validate_slug(p, "project");
    }
    if pw_opts.non_interactive {
        return Err(AppError::Validation(
            "pass -p <project> when running 'gitvault init' with --non-interactive".into(),
        ));
    }

    let existing = store.list_projects()?;
    let mut options = vec!["+ Create new project".to_owned()];
    options.extend(existing.iter().cloned());

    let choice = ui::select("Project:", &options, 0)?;
    if choice == 0 {
        return prompt_new_project_name(cwd);
    }
    Ok(existing[choice - 1].clone())
}

fn prompt_new_project_name(cwd: &Path) -> Result<String, AppError> {
    let suggested = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .map(sanitize_slug)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "my-app".to_owned());
    loop {
        let raw = ui::input("Project name", Some(&suggested))?;
        match validate_slug(&raw, "project") {
            Ok(valid) => return Ok(valid),
            Err(e) => ui::warn(&e.to_string()),
        }
    }
}

fn resolve_init_env(
    envs: &[String],
    flag: Option<&str>,
    pw_opts: PasswordOptions,
) -> Result<String, AppError> {
    if let Some(e) = flag {
        return validate_slug(e, "environment");
    }
    if pw_opts.non_interactive {
        return Err(AppError::Validation(
            "pass -e <environment> when running 'gitvault init' with --non-interactive".into(),
        ));
    }
    let mut options = envs.to_vec();
    options.push("+ Create custom environment".to_owned());
    let choice = ui::select("Default environment:", &options, 0)?;
    if choice != options.len() - 1 {
        return Ok(envs[choice].clone());
    }
    loop {
        let raw = ui::input("Environment name", None)?;
        match validate_slug(&raw, "environment") {
            Ok(valid) => return Ok(valid),
            Err(e) => ui::warn(&e.to_string()),
        }
    }
}

fn sanitize_slug(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(64)
        .collect()
}
