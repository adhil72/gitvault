//! Command-line definition (global flags and subcommands).
//!
//! Every command has `--help` with examples; `gitvault commands --json` prints
//! this whole tree in machine-readable form.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// GitVault: encrypted secrets stored in your own private GitHub repository.
///
/// Examples:
///   gitvault auth login --from-gh
///   gitvault vault create --name Personal
///   gitvault secret add STRIPE_KEY --environment production --value-stdin
///   gitvault run --environment production -- ./deploy.sh
#[derive(Debug, Parser)]
#[command(name = "gitvault", version, propagate_version = true)]
pub struct Cli {
    /// Print machine-readable JSON on stdout (human text goes to stderr).
    #[arg(long, global = true)]
    pub json: bool,

    /// Vault id to use (default: the vault chosen with `gitvault vault use`).
    #[arg(long, global = true, env = "GITVAULT_VAULT")]
    pub vault: Option<String>,

    /// Never prompt; fail with an error instead.
    #[arg(long, global = true)]
    pub non_interactive: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show version information.
    Version,
    /// Sign in to GitHub.
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },
    /// List, create and manage vaults.
    Vault {
        #[command(subcommand)]
        cmd: VaultCmd,
    },
    /// Unlock a vault (prints a session token with --export).
    Unlock(UnlockArgs),
    /// Lock a vault and wipe its key.
    Lock,
    /// Trust folders: this device unlocks by itself only inside them (and their subfolders).
    Trust {
        #[command(subcommand)]
        cmd: TrustCmd,
    },
    /// Trust devices so they unlock without the master password.
    Device {
        #[command(subcommand)]
        cmd: DeviceCmd,
    },
    /// Give GitHub users access to the vault's repository (they still need the master password).
    Collaborator {
        #[command(subcommand)]
        cmd: CollaboratorCmd,
    },
    /// Add, read, change and delete secrets.
    Secret {
        #[command(subcommand)]
        cmd: SecretCmd,
    },
    /// List environments with secret counts.
    Env {
        #[command(subcommand)]
        cmd: EnvCmd,
    },
    /// List tags with secret counts.
    Tag {
        #[command(subcommand)]
        cmd: ListOnly,
    },
    /// Import secrets from a .env file.
    Import {
        #[command(subcommand)]
        cmd: ImportCmd,
    },
    /// Export secrets as a .env file.
    Export {
        #[command(subcommand)]
        cmd: ExportCmd,
    },
    /// Generate passwords, passphrases and PINs.
    Generate {
        #[command(subcommand)]
        cmd: GenerateCmd,
    },
    /// Run a command with secrets injected as environment variables (values are never printed).
    Run(RunArgs),
    /// Fill a template with secret values.
    Inject(InjectArgs),
    /// Sync with GitHub.
    Sync {
        #[command(subcommand)]
        cmd: SyncCmd,
    },
    /// Show the local audit log.
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// Read or change settings.
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
    /// Print every command as JSON (for AI tools).
    Commands,
    /// Print the JSON Schema of an output type (or list the types).
    Schema { name: Option<String> },
    /// Print shell completions.
    Completions { shell: Shell },
    /// Start a Model Context Protocol server on stdio.
    Mcp,
}

#[derive(Debug, Subcommand)]
pub enum EnvCmd {
    /// List environments with secret counts.
    List,
    /// Print one environment's variables as JSON (for app loaders such as the Node package).
    ///
    /// Includes shared secrets (no environment). Unlocks with GITVAULT_PASSWORD when the vault
    /// is locked, so servers need no prompt.
    Export {
        /// Environment name, e.g. development or production.
        environment: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ListOnly {
    List,
}

#[derive(Debug, Subcommand)]
pub enum AuthCmd {
    /// Sign in. Without flags, uses the GitHub device flow.
    Login(LoginArgs),
    /// Forget the GitHub token.
    Logout,
    /// Show the signed-in account.
    Status,
    /// List the accounts of the GitHub CLI (`gh`).
    Accounts,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Read a personal access token from stdin.
    #[arg(long)]
    pub with_token: bool,
    /// Reuse the token of the GitHub CLI (`gh auth token`).
    #[arg(long)]
    pub from_gh: bool,
    /// Sign in as this account of the GitHub CLI (see `gitvault auth accounts`).
    #[arg(long, value_name = "LOGIN", conflicts_with_all = ["with_token", "from_gh"])]
    pub gh_user: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum VaultCmd {
    /// List the vaults on this device (add --remote to also look on GitHub).
    List {
        #[arg(long)]
        remote: bool,
    },
    /// Create a vault (and a private GitHub repository when signed in).
    Create(CreateVaultArgs),
    /// Choose the default vault.
    Use { id: String },
    /// Show one vault.
    Info,
    /// Clone a vault found on GitHub (owner/name).
    Connect { repo: String },
    /// Remove the local copy (GitHub is untouched).
    Forget { id: String },
    /// Check whether the vault's GitHub repository still exists.
    Remote,
    /// Create a new GitHub repository for a vault whose repository was deleted.
    Republish,
    /// Change the master password.
    Password(PasswordArgs),
    /// Create a new recovery key.
    Recovery(PasswordArgs),
}

#[derive(Debug, Args)]
pub struct CreateVaultArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    /// GitHub repository name (default: gitvault-<name>).
    #[arg(long)]
    pub repo_name: Option<String>,
    /// Do not create a GitHub repository.
    #[arg(long)]
    pub local_only: bool,
    #[command(flatten)]
    pub password: PasswordSource,
}

#[derive(Debug, Subcommand)]
pub enum TrustCmd {
    /// Trust a folder and everything under it (trusts this device first if needed).
    Add {
        /// The folder (default: the current one).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Only for these environments, e.g. --env development. Default: all of them.
        #[arg(long = "env")]
        environments: Vec<String>,
        #[command(flatten)]
        password: PasswordSource,
    },
    /// List the trusted folders.
    List,
    /// Stop trusting a folder.
    Remove { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub enum DeviceCmd {
    /// Trust this device (or, with --export, print a key for a server or CI job).
    ///
    /// Needs the master password once. A server then sets GITVAULT_DEVICE_KEY instead of a password.
    Trust {
        /// A name to recognise the device by, e.g. laptop or prod-server.
        #[arg(long)]
        name: String,
        /// Print a device key for another machine (shown once) instead of trusting this one.
        #[arg(long)]
        export: bool,
        #[command(flatten)]
        password: PasswordSource,
    },
    /// List the trusted devices of the vault.
    List,
    /// Stop trusting a device (needs the vault unlocked).
    Revoke {
        id: String,
        #[command(flatten)]
        password: PasswordSource,
    },
}

#[derive(Debug, Subcommand)]
pub enum CollaboratorCmd {
    /// List the collaborators and pending invitations of the vault's repository.
    List,
    /// Invite a GitHub user to the vault's repository (read-only unless --write).
    Invite {
        /// The GitHub username, e.g. octocat.
        login: String,
        /// Let them push secret changes too.
        #[arg(long)]
        write: bool,
    },
    /// Remove a collaborator, or cancel their pending invitation.
    Remove { login: String },
}

#[derive(Debug, Args)]
pub struct PasswordArgs {
    #[command(flatten)]
    pub password: PasswordSource,
}

/// Where to read the master password from.
#[derive(Debug, Args)]
pub struct PasswordSource {
    /// Read the master password from stdin instead of prompting.
    #[arg(long)]
    pub password_stdin: bool,
}

#[derive(Debug, Args)]
pub struct UnlockArgs {
    #[command(flatten)]
    pub password: PasswordSource,
    /// Unlock with a recovery key read from stdin.
    #[arg(long)]
    pub recovery_key_stdin: bool,
    /// Print the session token (`GITVAULT_SESSION`) for scripts.
    #[arg(long)]
    pub export: bool,
}

/// Where to read a secret value from (values are never accepted as arguments).
#[derive(Debug, Args, Default)]
pub struct ValueSource {
    /// Read the value from stdin.
    #[arg(long)]
    pub value_stdin: bool,
    /// Read the value from a file.
    #[arg(long)]
    pub from_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortArg {
    Updated,
    Name,
    Expiry,
}

#[derive(Debug, Args)]
pub struct FilterArgs {
    #[arg(long)]
    pub environment: Option<String>,
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SecretCmd {
    /// Add a secret. The value comes from stdin, a file, or a hidden prompt.
    Add(AddArgs),
    /// Show a secret (value masked unless --reveal / --raw).
    Get(GetArgs),
    /// List secrets.
    List(ListArgs),
    /// Change fields of a secret.
    Update(UpdateArgs),
    /// Set a new value.
    Rotate {
        secret: String,
        #[command(flatten)]
        value: ValueSource,
    },
    /// Delete a secret (restorable from history).
    Delete { secret: String },
    /// Show the version history.
    History { secret: String },
    /// Restore an old version.
    Restore { secret: String, commit: String },
    /// Search names, environments and tags.
    Search { text: String },
}

#[derive(Debug, Args)]
pub struct AddArgs {
    pub name: String,
    #[arg(long, default_value = "api_key")]
    pub kind: String,
    #[command(flatten)]
    pub filter: FilterArgs,
    #[arg(long)]
    pub notes: Option<String>,
    /// Expiry date, YYYY-MM-DD.
    #[arg(long)]
    pub expires: Option<String>,
    #[arg(long)]
    pub remind_days: Option<u32>,
    #[command(flatten)]
    pub value: ValueSource,
}

#[derive(Debug, Args)]
pub struct GetArgs {
    pub secret: String,
    /// Include the plaintext value (recorded in the audit log).
    #[arg(long)]
    pub reveal: bool,
    /// Print only the value (implies --reveal).
    #[arg(long)]
    pub raw: bool,
    /// Disambiguate when the name exists in several environments.
    #[arg(long)]
    pub environment: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub search: Option<String>,
    #[command(flatten)]
    pub filter: FilterArgs,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub favorites: bool,
    #[arg(long)]
    pub expiring_days: Option<u32>,
    #[arg(long, value_enum)]
    pub sort: Option<SortArg>,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    pub secret: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    /// Empty string clears it.
    #[arg(long)]
    pub environment: Option<String>,
    /// Replaces all tags (repeat the flag).
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long)]
    pub notes: Option<String>,
    /// YYYY-MM-DD, or empty to clear.
    #[arg(long)]
    pub expires: Option<String>,
    #[arg(long)]
    pub favorite: Option<bool>,
    #[command(flatten)]
    pub value: ValueSource,
}

#[derive(Debug, Subcommand)]
pub enum ImportCmd {
    /// Import a .env file.
    Env(ImportArgs),
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub environment: Option<String>,
    /// Replace values of secrets that already exist.
    #[arg(long)]
    pub overwrite: bool,
    /// Show what would happen without changing anything.
    #[arg(long)]
    pub preview: bool,
}

#[derive(Debug, Subcommand)]
pub enum ExportCmd {
    /// Print (or write) a .env file.
    Env {
        #[command(flatten)]
        filter: FilterArgs,
        /// Write to a file (mode 0600) instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum GenerateCmd {
    Password {
        #[arg(long, default_value_t = 20)]
        length: u32,
        #[arg(long)]
        no_uppercase: bool,
        #[arg(long)]
        no_lowercase: bool,
        #[arg(long)]
        no_digits: bool,
        #[arg(long)]
        no_symbols: bool,
        #[arg(long)]
        avoid_ambiguous: bool,
    },
    Passphrase {
        #[arg(long, default_value_t = 5)]
        words: u32,
        #[arg(long, default_value = "-")]
        separator: String,
        #[arg(long)]
        capitalize: bool,
    },
    Pin {
        #[arg(long, default_value_t = 6)]
        length: u32,
    },
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[command(flatten)]
    pub filter: FilterArgs,
    /// Capture output and replace secret values with [REDACTED].
    #[arg(long)]
    pub redact: bool,
    /// The command to run (after `--`).
    #[arg(trailing_var_arg = true, required = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct InjectArgs {
    /// Template with `{{ NAME }}` or `{{ gitvault://NAME }}` placeholders.
    #[arg(long = "in")]
    pub input: PathBuf,
    /// Output file (mode 0600).
    #[arg(long = "out")]
    pub output: PathBuf,
    #[command(flatten)]
    pub filter: FilterArgs,
}

#[derive(Debug, Subcommand)]
pub enum SyncCmd {
    /// Pull and push.
    Now,
    Status,
    /// List conflicted secrets.
    Conflicts,
    /// Keep one side of a conflict.
    Resolve {
        secret: String,
        #[arg(long, value_parser = ["local", "remote"])]
        keep: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuditCmd {
    List {
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: u32,
        /// Include events of every vault.
        #[arg(long)]
        all_vaults: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SettingsCmd {
    Get,
    /// Keys: auto_lock_minutes, clear_clipboard_seconds, hide_revealed_seconds, theme, auto_sync, github_client_id.
    Set {
        key: String,
        value: String,
    },
}
