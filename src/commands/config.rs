use std::{collections::BTreeMap, fs, path::Path};

use serde::Serialize;

use crate::{
    cli::{ConfigArgs, ConfigSubcmd, EnvSubcmd},
    commands::{
        config_interactive::config_interactive, current_dir, ensure_store_and_unlock,
        print_sync_outcome, EnvTarget, SecretVars,
    },
    crypto::VaultKey,
    domain::{mask_value, parse_dotenv, validate_slug, validate_var_key, ProjectConfig},
    drivers::{PasswordOptions, VaultStore},
    error::AppError,
    ui,
};

pub fn config(args: ConfigArgs, pw_opts: PasswordOptions) -> Result<(), AppError> {
    let cwd = current_dir()?;
    let store = VaultStore::open_default()?;
    let key = ensure_store_and_unlock(&store, pw_opts, true)?;

    let Some(subcommand) = args.subcommand else {
        if pw_opts.non_interactive {
            return Err(AppError::Validation(
                "specify a config subcommand in --non-interactive mode (e.g. 'gitvault config list')"
                    .into(),
            ));
        }
        return config_interactive(
            &store,
            &key,
            args.project.as_deref(),
            args.environment.as_deref(),
            &cwd,
        );
    };

    match subcommand {
        ConfigSubcmd::Env { cmd } => {
            let project = resolve_project_only(args.project.as_deref(), &cwd)?;
            config_env_cmd(&store, &key, &project, cmd, pw_opts)
        }
        other => {
            let (project, env) =
                ProjectConfig::resolve(args.project.as_deref(), args.environment.as_deref(), &cwd)?;
            let target = EnvTarget {
                store: &store,
                key: &key,
                project: &project,
                env: &env,
            };
            config_var_cmd(&target, other, pw_opts)
        }
    }
}

fn resolve_project_only(project_flag: Option<&str>, cwd: &Path) -> Result<String, AppError> {
    if let Some(p) = project_flag {
        return validate_slug(p, "project");
    }
    if let Some((_, cfg)) = ProjectConfig::find_in_ancestors(cwd)? {
        return Ok(cfg.project);
    }
    Err(AppError::NotFound(
        "no gitvault.json found here (run 'gitvault init' or pass -p <project>)".into(),
    ))
}

fn config_env_cmd(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    cmd: EnvSubcmd,
    pw_opts: PasswordOptions,
) -> Result<(), AppError> {
    match cmd {
        EnvSubcmd::List => env_list(store, key, project),
        EnvSubcmd::Create { name } => env_create(store, project, &name),
        EnvSubcmd::Delete { name, yes } => env_delete(store, project, &name, yes, pw_opts),
        EnvSubcmd::Clone { from, to } => env_clone(store, key, project, &from, &to),
    }
}

fn env_list(store: &VaultStore, key: &VaultKey, project: &str) -> Result<(), AppError> {
    let manifest = store.read_project(project)?;
    for env in &manifest.environments {
        let count = store.read_env_vars(key, project, env)?.len();
        println!("{env}\t{count} variable(s)");
    }
    Ok(())
}

fn env_create(store: &VaultStore, project: &str, name: &str) -> Result<(), AppError> {
    let env = validate_slug(name, "environment")?;
    if !store.ensure_environment(project, &env)? {
        ui::info(&format!(
            "Environment '{env}' already exists in '{project}'"
        ));
        return Ok(());
    }
    let res = store.commit_and_sync(&format!("env: create {project}/{env}"))?;
    print_sync_outcome(res, &format!("Created environment '{env}' in '{project}'"));
    Ok(())
}

fn env_delete(
    store: &VaultStore,
    project: &str,
    name: &str,
    yes: bool,
    pw_opts: PasswordOptions,
) -> Result<(), AppError> {
    let env = validate_slug(name, "environment")?;
    if !yes {
        if pw_opts.non_interactive {
            return Err(AppError::Validation(
                "pass --yes to delete an environment non-interactively".into(),
            ));
        }
        if !ui::confirm(
            &format!("Delete environment '{env}' from '{project}'?"),
            false,
        )? {
            return Ok(());
        }
    }
    store.delete_environment(project, &env)?;
    let res = store.commit_and_sync(&format!("env: delete {project}/{env}"))?;
    print_sync_outcome(
        res,
        &format!("Deleted environment '{env}' from '{project}'"),
    );
    Ok(())
}

fn env_clone(
    store: &VaultStore,
    key: &VaultKey,
    project: &str,
    from: &str,
    to: &str,
) -> Result<(), AppError> {
    let from = validate_slug(from, "environment")?;
    let to = validate_slug(to, "environment")?;
    let src = SecretVars(store.read_env_vars(key, project, &from)?);
    store.ensure_environment(project, &to)?;
    store.write_env_vars(key, project, &to, &src.0)?;
    let res = store.commit_and_sync(&format!("env: clone {project}/{from} -> {to}"))?;
    print_sync_outcome(
        res,
        &format!("Cloned {} variable(s) from '{from}' to '{to}'", src.0.len()),
    );
    Ok(())
}

fn config_var_cmd(
    t: &EnvTarget<'_>,
    cmd: ConfigSubcmd,
    pw_opts: PasswordOptions,
) -> Result<(), AppError> {
    match cmd {
        ConfigSubcmd::Set { items, value_stdin } => cmd_set(t, items, value_stdin, pw_opts),
        ConfigSubcmd::Get {
            key: var_name,
            json,
            mask,
        } => cmd_get(t, &var_name, json, mask),
        ConfigSubcmd::List { reveal, json } => cmd_list(t, reveal, json),
        ConfigSubcmd::Rm { keys } => cmd_rm(t, keys),
        ConfigSubcmd::Import {
            file,
            overwrite,
            delete_source,
        } => import_dotenv_file(t, &file, overwrite, delete_source),
        ConfigSubcmd::Env { .. } => unreachable!(),
    }
}

fn cmd_set(
    t: &EnvTarget<'_>,
    items: Vec<String>,
    value_stdin: bool,
    pw_opts: PasswordOptions,
) -> Result<(), AppError> {
    let _ = t.store.read_project(t.project)?;
    let _ = t.store.ensure_environment(t.project, t.env)?;
    let mut vars = SecretVars(t.store.read_env_vars(t.key, t.project, t.env)?);
    let updated = collect_set_updates(&mut vars.0, items, value_stdin, pw_opts)?;
    t.store.write_env_vars(t.key, t.project, t.env, &vars.0)?;
    let res = t
        .store
        .commit_and_sync(&format!("secrets: update {}/{}", t.project, t.env))?;
    print_sync_outcome(
        res,
        &format!("Saved {} in {} ({})", updated.join(", "), t.project, t.env),
    );
    Ok(())
}

fn collect_set_updates(
    vars: &mut BTreeMap<String, String>,
    items: Vec<String>,
    value_stdin: bool,
    pw_opts: PasswordOptions,
) -> Result<Vec<String>, AppError> {
    if value_stdin {
        if items.len() != 1 || items[0].contains('=') {
            return Err(AppError::Validation(
                "--value-stdin expects a single KEY name without '='".into(),
            ));
        }
        let k = validate_var_key(&items[0])?;
        let val = ui::read_stdin_secret()?;
        vars.insert(k.clone(), val.to_string());
        return Ok(vec![k]);
    }
    let mut updated = Vec::with_capacity(items.len());
    for item in items {
        let (k, v) = parse_set_item(&item, pw_opts)?;
        vars.insert(k.clone(), v);
        updated.push(k);
    }
    Ok(updated)
}

fn parse_set_item(item: &str, pw_opts: PasswordOptions) -> Result<(String, String), AppError> {
    if let Some((raw_k, raw_v)) = item.split_once('=') {
        return Ok((validate_var_key(raw_k)?, raw_v.to_owned()));
    }
    if pw_opts.non_interactive {
        return Err(AppError::Validation(format!(
            "missing '=VALUE' for '{item}' (or pass --value-stdin)"
        )));
    }
    let k = validate_var_key(item)?;
    let val = ui::password(&format!("Value for {k}"))?;
    Ok((k, val.to_string()))
}

fn cmd_get(t: &EnvTarget<'_>, var_name: &str, json: bool, mask: bool) -> Result<(), AppError> {
    let var_name = validate_var_key(var_name)?;
    let vars = SecretVars(t.store.read_env_vars(t.key, t.project, t.env)?);
    let val = vars.0.get(&var_name).ok_or_else(|| {
        AppError::NotFound(format!(
            "variable '{var_name}' not found in {} ({})",
            t.project, t.env
        ))
    })?;
    let shown = if mask { mask_value(val) } else { val.clone() };
    if json {
        #[derive(Serialize)]
        struct Entry<'a> {
            key: &'a str,
            value: &'a str,
        }
        let out = serde_json::to_string(&Entry {
            key: &var_name,
            value: &shown,
        })
        .map_err(|e| AppError::Validation(e.to_string()))?;
        println!("{out}");
    } else {
        println!("{shown}");
    }
    Ok(())
}

fn cmd_list(t: &EnvTarget<'_>, reveal: bool, json: bool) -> Result<(), AppError> {
    let vars = SecretVars(t.store.read_env_vars(t.key, t.project, t.env)?);
    if json {
        let output: BTreeMap<&str, String> = vars
            .0
            .iter()
            .map(|(k, v)| (k.as_str(), if reveal { v.clone() } else { mask_value(v) }))
            .collect();
        let text = serde_json::to_string_pretty(&output)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        println!("{text}");
    } else if vars.0.is_empty() {
        ui::info(&format!("No variables in {} ({})", t.project, t.env));
    } else {
        for (k, v) in &vars.0 {
            let display = if reveal { v.clone() } else { mask_value(v) };
            println!("{k}={display}");
        }
    }
    Ok(())
}

fn cmd_rm(t: &EnvTarget<'_>, keys: Vec<String>) -> Result<(), AppError> {
    let mut vars = SecretVars(t.store.read_env_vars(t.key, t.project, t.env)?);
    let mut removed = Vec::with_capacity(keys.len());
    for raw in keys {
        let k = validate_var_key(&raw)?;
        if vars.0.remove(&k).is_none() {
            return Err(AppError::NotFound(format!(
                "variable '{k}' not found in {} ({})",
                t.project, t.env
            )));
        }
        removed.push(k);
    }
    t.store.write_env_vars(t.key, t.project, t.env, &vars.0)?;
    let res = t
        .store
        .commit_and_sync(&format!("secrets: remove in {}/{}", t.project, t.env))?;
    print_sync_outcome(
        res,
        &format!(
            "Removed {} from {} ({})",
            removed.join(", "),
            t.project,
            t.env
        ),
    );
    Ok(())
}

pub fn import_dotenv_file(
    t: &EnvTarget<'_>,
    file: &Path,
    overwrite: bool,
    delete_source: bool,
) -> Result<(), AppError> {
    if !file.is_file() {
        return Err(AppError::NotFound(format!(
            "file not found: {}",
            file.display()
        )));
    }
    let _ = t.store.read_project(t.project)?;
    let _ = t.store.ensure_environment(t.project, t.env)?;
    let raw = zeroize::Zeroizing::new(fs::read_to_string(file)?);
    let incoming = SecretVars(parse_dotenv(&raw)?);
    let mut existing = SecretVars(t.store.read_env_vars(t.key, t.project, t.env)?);
    let (added, updated, skipped) = merge_env_vars(&mut existing.0, &incoming.0, overwrite);

    t.store
        .write_env_vars(t.key, t.project, t.env, &existing.0)?;
    let res = t
        .store
        .commit_and_sync(&format!("secrets: import {}/{}", t.project, t.env))?;
    print_sync_outcome(
        res,
        &format!(
            "Imported into {} ({}): {added} added, {updated} updated, {skipped} skipped",
            t.project, t.env
        ),
    );
    if delete_source {
        fs::remove_file(file)?;
        ui::success(&format!("Removed plaintext file '{}'", file.display()));
    }
    Ok(())
}

fn merge_env_vars(
    existing: &mut BTreeMap<String, String>,
    incoming: &BTreeMap<String, String>,
    overwrite: bool,
) -> (usize, usize, usize) {
    let (mut added, mut updated, mut skipped) = (0_usize, 0_usize, 0_usize);
    for (k, v) in incoming {
        match existing.get(k) {
            None => {
                existing.insert(k.clone(), v.clone());
                added += 1;
            }
            Some(_) if overwrite => {
                existing.insert(k.clone(), v.clone());
                updated += 1;
            }
            Some(_) => skipped += 1,
        }
    }
    (added, updated, skipped)
}
