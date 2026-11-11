//! `gitvault` command-line interface.
//!
//! The CLI only parses input, calls a service from `gitvault-core` through
//! `Services`, and renders the result. It contains no business logic.

mod app;
mod cli;
mod commands;
mod input;
mod output;
mod time;

use std::process::ExitCode;

use clap::Parser;
use gitvault_core::CoreError;
use gitvault_infra::{build_services, AppConfig};

use app::App;
use cli::{Cli, Command};
use commands::{organize::Facet, *};

const SESSION_ENV: &str = "GITVAULT_SESSION";
const TOKEN_ENV: &str = "GITVAULT_TOKEN";

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    match execute(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => report(&error, json),
    }
}

fn execute(cli: Cli) -> anyhow::Result<u8> {
    let services = build_services(&AppConfig::from_env("CLI", true))?;
    if let Ok(token) = std::env::var(SESSION_ENV) {
        services.vaults.import_session(&token)?;
    }
    sign_in_from_env(&services)?;
    let app = App::new(services, cli.json, cli.non_interactive, cli.vault);
    if let Command::Run(args) = &cli.command {
        return run::run(&app, args);
    }
    dispatch(&app, &cli.command)?;
    Ok(0)
}

/// Servers have no `gh` or browser: `GITVAULT_TOKEN` (a read-only GitHub token) signs them in.
fn sign_in_from_env(services: &gitvault_core::services::Services) -> anyhow::Result<()> {
    if services.auth.status().is_none() {
        if let Ok(token) = std::env::var(TOKEN_ENV) {
            services.auth.login_with_token(&token)?;
        }
    }
    Ok(())
}

fn dispatch(app: &App, command: &Command) -> anyhow::Result<()> {
    match command {
        Command::Version => meta::version(app),
        Command::Auth { cmd } => auth::run(app, cmd),
        Command::Vault { cmd } => vault::run(app, cmd),
        Command::Unlock(args) => vault::unlock(app, args),
        Command::Lock => vault::lock(app),
        Command::Device { cmd } => device::run(app, cmd),
        Command::Collaborator { cmd } => collaborator::run(app, cmd),
        Command::Trust { cmd } => trust::run(app, cmd),
        Command::Secret { cmd } => secret::run(app, cmd),
        Command::Env { cmd } => match cmd {
            cli::EnvCmd::List => organize::list(app, Facet::Environment),
            cli::EnvCmd::Export { environment } => load::export(app, environment),
        },
        Command::Tag { .. } => organize::list(app, Facet::Tag),
        Command::Import { cmd } => transfer::import(app, cmd),
        Command::Export { cmd } => transfer::export(app, cmd),
        Command::Generate { cmd } => generate::run(app, cmd),
        Command::Run(_) => Ok(()),
        Command::Inject(args) => run::inject(app, args),
        Command::Sync { cmd } => sync::run(app, cmd),
        Command::Audit { cmd } => audit::run(app, cmd),
        Command::Settings { cmd } => settings::run(app, cmd),
        Command::Commands => meta::commands(app),
        Command::Schema { name } => meta::schema(app, name.as_deref()),
        Command::Completions { shell } => {
            meta::completions(*shell);
            Ok(())
        }
        Command::Mcp => mcp::serve(app),
    }
}

/// Prints the error (JSON in machine mode) and returns its exit code.
fn report(error: &anyhow::Error, json: bool) -> ExitCode {
    let core = error.downcast_ref::<CoreError>();
    let (code, hint) = core.map_or(("E_INTERNAL", None), |e| (e.code().as_str(), e.hint()));
    let exit = core.map_or(1, |e| e.code().exit_code());
    let message = format!("{error:#}");
    if json {
        let body =
            serde_json::json!({ "error": { "code": code, "message": message, "hint": hint } });
        println!("{body}");
    } else {
        eprintln!("error: {message}");
        if let Some(hint) = hint {
            eprintln!("hint: {hint}");
        }
    }
    ExitCode::from(exit)
}
