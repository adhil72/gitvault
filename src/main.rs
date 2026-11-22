mod cli;
mod commands;
mod crypto;
mod domain;
mod drivers;
mod error;
mod ui;

use std::process::ExitCode;

use clap::Parser;

use crate::{
    cli::{Cli, Command},
    drivers::PasswordOptions,
    error::AppError,
};

fn dispatch(cli: Cli) -> Result<i32, AppError> {
    let pw_opts = PasswordOptions {
        password_stdin: cli.password_stdin,
        non_interactive: cli.non_interactive,
    };
    match cli.command {
        Command::Init(args) => commands::init(args, pw_opts).map(|()| 0),
        Command::Config(args) => commands::config(args, pw_opts).map(|()| 0),
        Command::Load(args) => commands::load(args, pw_opts),
        Command::Unload(args) => commands::unload(args).map(|()| 0),
        Command::ShellInit(args) => commands::shell_init(args).map(|()| 0),
        Command::Sync(args) => commands::sync(args).map(|()| 0),
        Command::Unlock(args) => commands::unlock(args, pw_opts).map(|()| 0),
        Command::Lock => commands::lock().map(|()| 0),
        Command::Projects(args) => commands::projects(args).map(|()| 0),
        Command::Doctor(args) => commands::doctor(args).map(|()| 0),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli) {
        Ok(code) => ExitCode::from(u8::try_from(code.clamp(0, 255)).unwrap_or(1)),
        Err(err) => {
            eprintln!("✗ {err}");
            if let Some(hint) = err.hint() {
                eprintln!("  Hint: {hint}");
            }
            ExitCode::from(u8::try_from(err.exit_code().clamp(1, 255)).unwrap_or(1))
        }
    }
}
