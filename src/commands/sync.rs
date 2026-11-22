use serde::Serialize;

use crate::{
    cli::{ProjectsArgs, SyncArgs},
    drivers::{GitDriver, Prerequisites, VaultStore},
    error::AppError,
    ui,
};

pub fn sync(args: SyncArgs) -> Result<(), AppError> {
    Prerequisites::ensure_ready()?;
    let store = VaultStore::open_default()?;
    if !store.is_initialized() {
        store.ensure_initialized_from_existing()?;
    }
    if args.status {
        let status = GitDriver::status(&store.paths.store_dir)?;
        println!("Branch:  {}", status.branch);
        println!(
            "Remote:  {}",
            status.remote_url.as_deref().unwrap_or("(local only)")
        );
        println!("Dirty:   {}", status.dirty_files);
        println!("Ahead:   {}", status.ahead);
        println!("Behind:  {}", status.behind);
        return Ok(());
    }
    let (pulled, pushed) = store.sync_strict()?;
    ui::success(&format!(
        "Vault synced with GitHub (pulled changes: {pulled}, pushed changes: {pushed})"
    ));
    Ok(())
}

pub fn projects(args: ProjectsArgs) -> Result<(), AppError> {
    Prerequisites::ensure_ready()?;
    let store = VaultStore::open_default()?;
    if !store.is_initialized() {
        store.ensure_initialized_from_existing()?;
    } else {
        store.pull_best_effort();
    }

    #[derive(Serialize)]
    struct ProjectInfo {
        name: String,
        environments: Vec<String>,
    }

    let names = store.list_projects()?;
    let mut infos = Vec::with_capacity(names.len());
    for name in names {
        let manifest = store.read_project(&name)?;
        infos.push(ProjectInfo {
            name: manifest.name,
            environments: manifest.environments,
        });
    }

    if args.json {
        let out = serde_json::to_string_pretty(&infos)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        println!("{out}");
    } else if infos.is_empty() {
        ui::info("No projects yet. Run 'gitvault init' inside a project folder.");
    } else {
        for p in infos {
            println!("{}\t[{}]", p.name, p.environments.join(", "));
        }
    }
    Ok(())
}
