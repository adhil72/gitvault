use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    commands::{config::import_dotenv_file, print_sync_outcome, EnvTarget, SecretVars},
    crypto::VaultKey,
    domain::{mask_value, validate_slug, validate_var_key, ProjectConfig},
    drivers::VaultStore,
    error::AppError,
    ui,
};

pub fn config_interactive(
    store: &VaultStore,
    key: &VaultKey,
    project_flag: Option<&str>,
    env_flag: Option<&str>,
    cwd: &Path,
) -> Result<(), AppError> {
    let mut project = pick_initial_project(store, project_flag, cwd)?;
    if let Some(env_name) = env_flag {
        let env = validate_slug(env_name, "environment")?;
        let _ = store.ensure_environment(&project, &env)?;
        return interactive_env_loop(store, key, &project, &env);
    }

    loop {
        ui::header(&format!("GitVault Config — Project: {project}"));
        let options = [
            "Config variables",
            "Manage environments",
            "Import & migrate .env file",
            "Switch project",
            "Sync with GitHub",
            "Exit",
        ]
        .map(str::to_owned)
        .to_vec();
        match ui::select("What would you like to do?", &options, 0)? {
            0 => open_env_variables_menu(store, key, &project)?,
            1 => interactive_manage_envs(store, key, &project)?,
            2 => open_import_menu(store, key, &project)?,
            3 => project = pick_project_menu(store)?,
            4 => run_interactive_sync(store),
            _ => break,
        }
    }
    Ok(())
}

fn open_env_variables_menu(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
) -> Result<(), AppError> {
    if let Some(env) = select_or_create_env(store, key, project)? {
        interactive_env_loop(store, key, project, &env)?;
    }
    Ok(())
}

fn open_import_menu(store: &VaultStore, key: &VaultKey, project: &str) -> Result<(), AppError> {
    if let Some(env) = select_or_create_env(store, key, project)? {
        interactive_import_prompt(store, key, project, &env)?;
    }
    Ok(())
}

fn run_interactive_sync(store: &VaultStore) {
    match store.sync_strict() {
        Ok((pulled, pushed)) => {
            ui::success(&format!(
                "Sync complete (pulled: {pulled}, pushed: {pushed})"
            ));
        }
        Err(e) => ui::warn(&e.to_string()),
    }
}

fn pick_initial_project(
    store: &VaultStore,
    flag: Option<&str>,
    cwd: &Path,
) -> Result<String, AppError> {
    if let Some(p) = flag {
        let slug = validate_slug(p, "project")?;
        let _ = store.ensure_project(&slug, None)?;
        return Ok(slug);
    }
    if let Some((_, cfg)) = ProjectConfig::find_in_ancestors(cwd)? {
        if store.read_project(&cfg.project).is_ok() {
            return Ok(cfg.project);
        }
    }
    pick_project_menu(store)
}

fn pick_project_menu(store: &VaultStore) -> Result<String, AppError> {
    let projects = store.list_projects()?;
    if projects.is_empty() {
        let raw = ui::input("No projects yet. Enter new project name", Some("my-app"))?;
        return create_project_from_input(store, &raw);
    }
    let mut options = projects.clone();
    options.push("+ Create new project".to_owned());
    let idx = ui::select("Select project:", &options, 0)?;
    if idx == options.len() - 1 {
        let raw = ui::input("New project name", None)?;
        return create_project_from_input(store, &raw);
    }
    Ok(projects[idx].clone())
}

fn create_project_from_input(store: &VaultStore, raw: &str) -> Result<String, AppError> {
    let slug = validate_slug(raw, "project")?;
    let _ = store.ensure_project(&slug, None)?;
    let _ = store.commit_and_sync(&format!("project: create {slug}"))?;
    Ok(slug)
}

fn select_or_create_env(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
) -> Result<Option<String>, AppError> {
    let manifest = store.read_project(project)?;
    let mut options = Vec::with_capacity(manifest.environments.len() + 2);
    for env in &manifest.environments {
        let count = store.read_env_vars(key, project, env)?.len();
        options.push(format!("{env} ({count} variable(s))"));
    }
    let create_idx = options.len();
    options.push("+ Create new environment".to_owned());
    let back_idx = options.len();
    options.push("← Back".to_owned());

    let choice = ui::select("Select or create environment:", &options, 0)?;
    if choice == back_idx {
        return Ok(None);
    }
    if choice == create_idx {
        return prompt_create_env(store, project).map(Some);
    }
    Ok(Some(manifest.environments[choice].clone()))
}

fn prompt_create_env(store: &VaultStore, project: &str) -> Result<String, AppError> {
    loop {
        let raw = ui::input("New environment name", None)?;
        match validate_slug(&raw, "environment") {
            Ok(env) => {
                if store.ensure_environment(project, &env)? {
                    let res = store.commit_and_sync(&format!("env: create {project}/{env}"))?;
                    print_sync_outcome(res, &format!("Created environment '{env}'"));
                }
                return Ok(env);
            }
            Err(e) => ui::warn(&e.to_string()),
        }
    }
}

fn interactive_env_loop(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    env: &str,
) -> Result<(), AppError> {
    let t = EnvTarget {
        store,
        key,
        project,
        env,
    };
    loop {
        let mut vars = SecretVars(store.read_env_vars(key, project, env)?);
        ui::header(&format!("{project} › {env} ({} variable(s))", vars.0.len()));
        let options = [
            "Add / Update variable",
            "View variables",
            "Delete variable",
            "Import from .env file",
            "Copy variables to another environment",
            "← Back",
        ]
        .map(str::to_owned)
        .to_vec();
        match ui::select("Action:", &options, 0)? {
            0 => interactive_add_var(&t, &mut vars.0)?,
            1 => interactive_view_vars(&vars.0)?,
            2 => interactive_delete_var(&t, &mut vars.0)?,
            3 => interactive_import_prompt(store, key, project, env)?,
            4 => interactive_copy_vars(&t, &vars.0)?,
            _ => break,
        }
    }
    Ok(())
}

fn interactive_add_var(
    t: &EnvTarget<'_>,
    vars: &mut BTreeMap<String, String>,
) -> Result<(), AppError> {
    let raw_key = ui::input("Variable name", None)?;
    let var_key = match validate_var_key(&raw_key) {
        Ok(k) => k,
        Err(e) => {
            ui::warn(&e.to_string());
            return Ok(());
        }
    };
    if vars.contains_key(&var_key)
        && !ui::confirm(&format!("'{var_key}' exists. Overwrite?"), true)?
    {
        return Ok(());
    }
    let val = ui::password(&format!("Value for {var_key}"))?;
    vars.insert(var_key.clone(), val.to_string());
    t.store.write_env_vars(t.key, t.project, t.env, vars)?;
    let res = t
        .store
        .commit_and_sync(&format!("secrets: set {}/{}", t.project, t.env))?;
    print_sync_outcome(res, &format!("Saved '{var_key}'"));
    Ok(())
}

fn interactive_view_vars(vars: &BTreeMap<String, String>) -> Result<(), AppError> {
    if vars.is_empty() {
        ui::info("No variables stored yet.");
        return Ok(());
    }
    for (k, v) in vars {
        eprintln!("  {k} = {}", mask_value(v));
    }
    if ui::confirm("Reveal plaintext values?", false)? {
        for (k, v) in vars {
            eprintln!("  {k} = {v}");
        }
    }
    Ok(())
}

fn interactive_delete_var(
    t: &EnvTarget<'_>,
    vars: &mut BTreeMap<String, String>,
) -> Result<(), AppError> {
    if vars.is_empty() {
        ui::info("No variables to delete.");
        return Ok(());
    }
    let mut keys: Vec<String> = vars.keys().cloned().collect();
    keys.push("← Cancel".to_owned());
    let idx = ui::select("Select variable to delete:", &keys, keys.len() - 1)?;
    if idx == keys.len() - 1 {
        return Ok(());
    }
    let chosen = &keys[idx];
    vars.remove(chosen);
    t.store.write_env_vars(t.key, t.project, t.env, vars)?;
    let res = t
        .store
        .commit_and_sync(&format!("secrets: delete in {}/{}", t.project, t.env))?;
    print_sync_outcome(res, &format!("Deleted '{chosen}'"));
    Ok(())
}

fn interactive_copy_vars(
    t: &EnvTarget<'_>,
    vars: &BTreeMap<String, String>,
) -> Result<(), AppError> {
    if vars.is_empty() {
        ui::info("No variables to copy.");
        return Ok(());
    }
    let Some(target_env) = select_or_create_env(t.store, t.key, t.project)? else {
        return Ok(());
    };
    if target_env == t.env {
        ui::warn("Source and target environments are the same.");
        return Ok(());
    }
    let mut target_vars = SecretVars(t.store.read_env_vars(t.key, t.project, &target_env)?);
    let overwrite = ui::confirm("Overwrite keys that already exist in target?", false)?;
    let copied = copy_matching_keys(vars, &mut target_vars.0, overwrite);
    t.store
        .write_env_vars(t.key, t.project, &target_env, &target_vars.0)?;
    let res = t.store.commit_and_sync(&format!(
        "secrets: copy {}/{} -> {target_env}",
        t.project, t.env
    ))?;
    print_sync_outcome(
        res,
        &format!("Copied {copied} variable(s) to '{target_env}'"),
    );
    Ok(())
}

fn copy_matching_keys(
    src: &BTreeMap<String, String>,
    dst: &mut BTreeMap<String, String>,
    overwrite: bool,
) -> usize {
    let mut copied = 0_usize;
    for (k, v) in src {
        if !dst.contains_key(k) || overwrite {
            dst.insert(k.clone(), v.clone());
            copied += 1;
        }
    }
    copied
}

fn interactive_manage_envs(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
) -> Result<(), AppError> {
    loop {
        let manifest = store.read_project(project)?;
        print_env_summary(store, key, project, &manifest.environments)?;
        let options = [
            "Create environment",
            "Clone environment",
            "Delete environment",
            "← Back",
        ]
        .map(str::to_owned)
        .to_vec();
        match ui::select("Manage environments:", &options, 3)? {
            0 => {
                let _ = prompt_create_env(store, project)?;
            }
            1 => interactive_clone_env(store, key, project, &manifest.environments)?,
            2 => interactive_delete_env(store, project, &manifest.environments)?,
            _ => break,
        }
    }
    Ok(())
}

fn print_env_summary(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    envs: &[String],
) -> Result<(), AppError> {
    ui::header(&format!("Environments in '{project}'"));
    for env in envs {
        let count = store.read_env_vars(key, project, env)?.len();
        eprintln!("  • {env} ({count} variable(s))");
    }
    Ok(())
}

fn interactive_clone_env(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    envs: &[String],
) -> Result<(), AppError> {
    if envs.is_empty() {
        return Ok(());
    }
    let src_idx = ui::select("Source environment:", envs, 0)?;
    let src_env = &envs[src_idx];
    let raw = ui::input("Target environment name", None)?;
    let dst_env = validate_slug(&raw, "environment")?;
    let vars = SecretVars(store.read_env_vars(key, project, src_env)?);
    store.ensure_environment(project, &dst_env)?;
    store.write_env_vars(key, project, &dst_env, &vars.0)?;
    let res = store.commit_and_sync(&format!("env: clone {project}/{src_env} -> {dst_env}"))?;
    print_sync_outcome(res, &format!("Cloned '{src_env}' to '{dst_env}'"));
    Ok(())
}

fn interactive_delete_env(
    store: &VaultStore,
    project: &str,
    envs: &[String],
) -> Result<(), AppError> {
    let mut choices = envs.to_vec();
    choices.push("← Cancel".to_owned());
    let idx = ui::select("Delete which environment?", &choices, choices.len() - 1)?;
    if idx == choices.len() - 1 {
        return Ok(());
    }
    let target = &choices[idx];
    if ui::confirm(&format!("Permanently delete '{target}'?"), false)? {
        store.delete_environment(project, target)?;
        let res = store.commit_and_sync(&format!("env: delete {project}/{target}"))?;
        print_sync_outcome(res, &format!("Deleted '{target}'"));
    }
    Ok(())
}

fn interactive_import_prompt(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    env: &str,
) -> Result<(), AppError> {
    let path_str = ui::input("Path to .env file", Some(".env"))?;
    let path = PathBuf::from(path_str);
    let overwrite = ui::confirm("Overwrite existing variables with same name?", true)?;
    let delete_source = ui::confirm("Delete plaintext .env file after encrypted import?", true)?;
    let target = EnvTarget {
        store,
        key,
        project,
        env,
    };
    import_dotenv_file(&target, &path, overwrite, delete_source)
}
