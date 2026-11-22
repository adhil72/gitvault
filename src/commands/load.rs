use std::{collections::BTreeMap, io::IsTerminal, process::Command};

use crate::{
    cli::{LoadArgs, ShellInitArgs, UnloadArgs},
    commands::{current_dir, SecretVars},
    domain::{format_shell_exports, format_shell_unsets, ProjectConfig, ShellKind},
    drivers::{PasswordOptions, Prerequisites, VaultStore},
    error::AppError,
    ui,
};

const POSIX_SHELL_INIT: &str = r#"gitvault() {
  if [ "${1:-}" = "load" ]; then
    shift
    for arg in "$@"; do
      case "$arg" in
        --export|--json|-h|--help|--)
          command gitvault load "$@"
          return $?
          ;;
      esac
    done
    local out
    out="$(command gitvault load "$@" --export)" || return $?
    eval "$out"
  elif [ "${1:-}" = "unload" ]; then
    shift
    for arg in "$@"; do
      case "$arg" in
        --export|-h|--help)
          command gitvault unload "$@"
          return $?
          ;;
      esac
    done
    local out
    out="$(command gitvault unload "$@" --export)" || return $?
    eval "$out"
  else
    command gitvault "$@"
  fi
}
gv() {
  gitvault "$@"
}"#;

const FISH_SHELL_INIT: &str = r#"function gitvault
  if test (count $argv) -ge 1 -a "$argv[1]" = "load"
    set -l rest $argv[2..-1]
    for arg in $rest
      switch $arg
        case --export --json -h --help --
          command gitvault load $rest
          return $status
      end
    end
    set -l out (command gitvault load $rest --export --shell fish | string collect)
    or return $status
    eval $out
  else if test (count $argv) -ge 1 -a "$argv[1]" = "unload"
    set -l rest $argv[2..-1]
    for arg in $rest
      switch $arg
        case --export -h --help
          command gitvault unload $rest
          return $status
      end
    end
    set -l out (command gitvault unload $rest --export --shell fish | string collect)
    or return $status
    eval $out
  else
    command gitvault $argv
  end
end
function gv
  gitvault $argv
end"#;

pub fn load(args: LoadArgs, pw_opts: PasswordOptions) -> Result<i32, AppError> {
    Prerequisites::ensure_ready()?;
    let cwd = current_dir()?;
    let (project, env) =
        ProjectConfig::resolve(args.project.as_deref(), args.environment.as_deref(), &cwd)?;

    let store = VaultStore::open_default()?;
    if !store.is_initialized() {
        store.ensure_initialized_from_existing()?;
    }
    if !args.no_sync {
        store.pull_best_effort();
    }

    let key = store.unlock(pw_opts, None, false)?;
    let vars = SecretVars(store.read_env_vars(&key, &project, &env)?);

    if !args.command.is_empty() {
        return run_with_env(&args.command, &vars.0);
    }
    if args.export {
        return print_shell_exports(&project, &env, &vars.0, args.shell.as_deref());
    }
    if args.json {
        return print_json_vars(&vars.0);
    }
    spawn_subshell(&project, &env, &vars.0, pw_opts)
}

fn run_with_env(command: &[String], vars: &BTreeMap<String, String>) -> Result<i32, AppError> {
    let status = Command::new(&command[0])
        .args(&command[1..])
        .envs(vars)
        .status()
        .map_err(|e| AppError::Validation(format!("failed to start '{}': {e}", command[0])))?;
    Ok(status.code().unwrap_or(1))
}

fn print_shell_exports(
    project: &str,
    env: &str,
    vars: &BTreeMap<String, String>,
    shell_flag: Option<&str>,
) -> Result<i32, AppError> {
    let shell = ShellKind::detect(shell_flag);
    let script = format_shell_exports(project, env, vars, shell)?;
    print!("{script}");
    ui::success(&format!(
        "Loaded {} variable(s) for {project} ({env})",
        vars.len()
    ));
    Ok(0)
}

fn print_json_vars(vars: &BTreeMap<String, String>) -> Result<i32, AppError> {
    let json =
        serde_json::to_string_pretty(vars).map_err(|e| AppError::Validation(e.to_string()))?;
    println!("{json}");
    Ok(0)
}

fn spawn_subshell(
    project: &str,
    env: &str,
    vars: &BTreeMap<String, String>,
    pw_opts: PasswordOptions,
) -> Result<i32, AppError> {
    if pw_opts.non_interactive || !std::io::stdin().is_terminal() {
        return Err(AppError::Validation(
            "pass '-- <command>', '--export', or '--json' when loading non-interactively".into(),
        ));
    }
    let shell_bin = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_owned());
    ui::success(&format!(
        "Loaded {} variable(s) for {project} ({env}) into subshell (exit to wipe)",
        vars.len()
    ));
    ui::info(&ui::dim(
        "Tip: add 'eval \"$(gitvault shell-init)\"' to your shell rc so 'gitvault load' updates your current shell directly.",
    ));
    let status = Command::new(&shell_bin)
        .envs(vars)
        .env("GITVAULT_ACTIVE_PROJECT", project)
        .env("GITVAULT_ACTIVE_ENV", env)
        .status()
        .map_err(|e| AppError::Validation(format!("failed to start shell '{shell_bin}': {e}")))?;
    Ok(status.code().unwrap_or(1))
}

pub fn unload(args: UnloadArgs) -> Result<(), AppError> {
    let loaded = std::env::var("GITVAULT_LOADED_KEYS").unwrap_or_default();
    if loaded.trim().is_empty() {
        ui::info("No gitvault variables active in this shell.");
        return Ok(());
    }
    if args.export {
        let shell = ShellKind::detect(args.shell.as_deref());
        let script = format_shell_unsets(&loaded, shell)?;
        print!("{script}");
        ui::success("Unloaded gitvault variables from shell");
    } else {
        ui::info("Run: eval \"$(gitvault unload --export)\"  (or install 'eval \"$(gitvault shell-init)\"')");
    }
    Ok(())
}

pub fn shell_init(args: ShellInitArgs) -> Result<(), AppError> {
    match ShellKind::detect(args.shell.as_deref()) {
        ShellKind::Posix => println!("{POSIX_SHELL_INIT}"),
        ShellKind::Fish => println!("{FISH_SHELL_INIT}"),
    }
    Ok(())
}
