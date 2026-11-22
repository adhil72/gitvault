use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    cli::DoctorArgs,
    commands::current_dir,
    domain::ProjectConfig,
    drivers::{GitDriver, PrereqReport, Prerequisites, VaultStore},
    error::AppError,
    ui,
};

#[derive(Serialize)]
struct DoctorOutput<'a> {
    git: Option<&'a str>,
    gh: Option<&'a str>,
    gh_authenticated: bool,
    gh_user: Option<&'a str>,
    store_dir: String,
    vault_initialized: bool,
    vault_unlocked: bool,
    remote: Option<String>,
    project_config: Option<ProjectConfig>,
}

pub fn doctor(args: DoctorArgs) -> Result<(), AppError> {
    let report = Prerequisites::diagnose();
    let store = VaultStore::open_default()?;
    let initialized = store.is_initialized();
    let unlocked = store.is_unlocked();
    let remote = initialized
        .then(|| GitDriver::remote_url(&store.paths.store_dir))
        .flatten();
    let cwd_config = current_dir()
        .ok()
        .and_then(|d| ProjectConfig::find_in_ancestors(&d).ok().flatten());

    if args.json {
        print_doctor_json(&report, &store, initialized, unlocked, remote, cwd_config)?;
    } else {
        print_doctor_human(
            &report,
            &store.paths.store_dir,
            initialized,
            unlocked,
            remote.as_deref(),
            cwd_config.as_ref(),
        );
    }

    if report.git_version.is_none() || report.gh_version.is_none() || !report.gh_auth_ok {
        return Prerequisites::ensure_ready();
    }
    Ok(())
}

fn print_doctor_json(
    report: &PrereqReport,
    store: &VaultStore,
    initialized: bool,
    unlocked: bool,
    remote: Option<String>,
    cwd_config: Option<(PathBuf, ProjectConfig)>,
) -> Result<(), AppError> {
    let payload = DoctorOutput {
        git: report.git_version.as_deref(),
        gh: report.gh_version.as_deref(),
        gh_authenticated: report.gh_auth_ok,
        gh_user: report.gh_user.as_deref(),
        store_dir: store.paths.store_dir.display().to_string(),
        vault_initialized: initialized,
        vault_unlocked: unlocked,
        remote,
        project_config: cwd_config.map(|(_, c)| c),
    };
    let out =
        serde_json::to_string_pretty(&payload).map_err(|e| AppError::Validation(e.to_string()))?;
    println!("{out}");
    Ok(())
}

fn print_doctor_human(
    report: &PrereqReport,
    store_dir: &Path,
    initialized: bool,
    unlocked: bool,
    remote: Option<&str>,
    cwd_config: Option<&(PathBuf, ProjectConfig)>,
) {
    print_prereq_lines(report);
    ui::info(&format!(
        "Vault store: {} ({}, {})",
        store_dir.display(),
        if initialized {
            "initialized"
        } else {
            "not initialized"
        },
        if unlocked { "unlocked" } else { "locked" }
    ));
    if let Some(url) = remote {
        ui::info(&format!("Vault remote: {url}"));
    }
    if let Some((path, cfg)) = cwd_config {
        ui::info(&format!(
            "Project config: {} ({}/{})",
            path.display(),
            cfg.project,
            cfg.environment
        ));
    }
}

fn print_prereq_lines(report: &PrereqReport) {
    match &report.git_version {
        Some(v) => ui::success(&format!("git: {v}")),
        None => ui::warn("git: missing (install from https://git-scm.com)"),
    }
    match &report.gh_version {
        Some(v) => ui::success(&format!("gh:  {v}")),
        None => ui::warn("gh:  missing (install from https://cli.github.com)"),
    }
    if report.gh_auth_ok {
        let user = report.gh_user.as_deref().unwrap_or("authenticated");
        ui::success(&format!("gh auth: signed in as {user}"));
    } else {
        ui::warn(&format!(
            "gh auth: not signed in ({})",
            report
                .gh_auth_message
                .as_deref()
                .unwrap_or("run 'gh auth login'")
        ));
    }
}
