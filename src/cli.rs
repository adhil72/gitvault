use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// GitVault v2: zero-.env encrypted secrets CLI powered exclusively by `git` and `gh`.
#[derive(Debug, Parser)]
#[command(
    name = "gitvault",
    version,
    about = "Encrypted project secrets stored safely in a private GitHub repo via git and gh — zero .env files.",
    propagate_version = true
)]
pub struct Cli {
    /// Never prompt for input; fail with an actionable error instead.
    #[arg(long, global = true)]
    pub non_interactive: bool,

    /// Read the master password from stdin instead of prompting.
    #[arg(long, global = true)]
    pub password_stdin: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Link or create a project in the encrypted vault and write gitvault.json (no .env file).
    Init(InitArgs),

    /// Interactively configure variables and environments, or run direct subcommands.
    Config(ConfigArgs),

    /// Load decrypted secrets into memory, shell session, JSON, or a child process (no .env file).
    Load(LoadArgs),

    /// Remove secrets previously loaded into the current shell session.
    Unload(UnloadArgs),

    /// Print shell integration snippet so `gitvault load` updates your active shell in RAM.
    #[command(name = "shell-init")]
    ShellInit(ShellInitArgs),

    /// Pull and push encrypted vault changes via git and gh.
    Sync(SyncArgs),

    /// Unlock the vault and cache the local session key (or change password).
    Unlock(UnlockArgs),

    /// Wipe the local unlocked session key cache.
    Lock,

    /// List all projects and environments stored in the vault.
    Projects(ProjectsArgs),

    /// Verify git, gh, GitHub authentication, and vault repository health.
    Doctor(DoctorArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Project name (skips interactive project selection).
    #[arg(short = 'p', long)]
    pub project: Option<String>,

    /// Default environment for this directory (skips environment selection).
    #[arg(short = 'e', long)]
    pub environment: Option<String>,

    /// Skip syncing with GitHub remote during initialization.
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// Target project (defaults to gitvault.json).
    #[arg(short = 'p', long, global = true)]
    pub project: Option<String>,

    /// Target environment (defaults to gitvault.json or interactive selection).
    #[arg(short = 'e', long, global = true)]
    pub environment: Option<String>,

    #[command(subcommand)]
    pub subcommand: Option<ConfigSubcmd>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcmd {
    /// Set one or more variables (KEY=VALUE, or KEY with --value-stdin / interactive prompt).
    Set {
        /// Variables as KEY=VALUE (or single KEY when using --value-stdin or interactive prompt).
        #[arg(required = true)]
        items: Vec<String>,

        /// Read a single variable's value from stdin.
        #[arg(long)]
        value_stdin: bool,
    },

    /// Print the value of a single variable.
    Get {
        /// Variable name.
        key: String,

        /// Output as JSON {"key": ..., "value": ...}.
        #[arg(long)]
        json: bool,

        /// Print masked preview instead of the raw value.
        #[arg(long)]
        mask: bool,
    },

    /// List variables in an environment (masked by default).
    List {
        /// Reveal plaintext values in the listing.
        #[arg(long)]
        reveal: bool,

        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Delete one or more variables from an environment.
    Rm {
        /// Variable names to delete.
        #[arg(required = true)]
        keys: Vec<String>,
    },

    /// Migrate variables from an existing legacy .env file into the encrypted vault.
    Import {
        /// Path to the .env file to import.
        file: PathBuf,

        /// Overwrite existing keys with the same name.
        #[arg(long)]
        overwrite: bool,

        /// Delete the plaintext .env file after successful encrypted import.
        #[arg(long)]
        delete_source: bool,
    },

    /// Manage environments within a project.
    Env {
        #[command(subcommand)]
        cmd: EnvSubcmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum EnvSubcmd {
    /// List environments for the project.
    List,

    /// Create a new environment in the project.
    Create { name: String },

    /// Delete an environment and its encrypted file.
    Delete {
        name: String,

        /// Skip confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Copy all variables from one environment into another.
    Clone { from: String, to: String },
}

#[derive(Debug, Args)]
pub struct LoadArgs {
    /// Override project from gitvault.json.
    #[arg(short = 'p', long)]
    pub project: Option<String>,

    /// Override environment from gitvault.json.
    #[arg(short = 'e', long)]
    pub environment: Option<String>,

    /// Print shell export statements to stdout (use with `eval`).
    #[arg(long, conflicts_with = "json")]
    pub export: bool,

    /// Print decrypted variables as a JSON object on stdout.
    #[arg(long, conflicts_with = "export")]
    pub json: bool,

    /// Shell syntax for --export (bash, zsh, fish).
    #[arg(long)]
    pub shell: Option<String>,

    /// Do not pull from GitHub before loading; read local encrypted store only.
    #[arg(long)]
    pub no_sync: bool,

    /// Command and arguments to run with decrypted variables injected in RAM (after `--`).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UnloadArgs {
    /// Print shell unset statements to stdout (use with `eval`).
    #[arg(long)]
    pub export: bool,

    /// Shell syntax for --export (bash, zsh, fish).
    #[arg(long)]
    pub shell: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShellInitArgs {
    /// Target shell (bash, zsh, or fish). Auto-detected from $SHELL when omitted.
    pub shell: Option<String>,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Show sync status without pulling or pushing.
    #[arg(long)]
    pub status: bool,
}

#[derive(Debug, Args)]
pub struct UnlockArgs {
    /// Unlock using a recovery key instead of the master password.
    #[arg(long)]
    pub recovery_key: Option<String>,

    /// Change the master password after unlocking.
    #[arg(long)]
    pub change_password: bool,
}

#[derive(Debug, Args)]
pub struct ProjectsArgs {
    /// Output as JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Output as JSON.
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_flags_after_subcommand() {
        let cli = Cli::try_parse_from([
            "gitvault",
            "config",
            "set",
            "DATABASE_URL=postgres://localhost/app",
            "-p",
            "shop-api",
            "-e",
            "development",
        ])
        .unwrap();
        match cli.command {
            Command::Config(cfg) => {
                assert_eq!(cfg.project.as_deref(), Some("shop-api"));
                assert_eq!(cfg.environment.as_deref(), Some("development"));
                assert!(matches!(cfg.subcommand, Some(ConfigSubcmd::Set { .. })));
            }
            _ => panic!("expected Config"),
        }
    }

    #[test]
    fn parses_load_with_flags_and_command() {
        let cli = Cli::try_parse_from([
            "gitvault",
            "load",
            "-p",
            "shop-api",
            "-e",
            "production",
            "--",
            "npm",
            "run",
            "dev",
        ])
        .unwrap();
        match cli.command {
            Command::Load(load) => {
                assert_eq!(load.project.as_deref(), Some("shop-api"));
                assert_eq!(load.environment.as_deref(), Some("production"));
                assert_eq!(load.command, vec!["npm", "run", "dev"]);
            }
            _ => panic!("expected Load"),
        }
    }
}
