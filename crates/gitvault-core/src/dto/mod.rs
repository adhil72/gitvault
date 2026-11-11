//! Serializable types that cross the UI / CLI boundary.
//!
//! These are the single source of truth: TypeScript types are generated from
//! them with `ts-rs` (`cargo test export_bindings`) and the CLI publishes JSON
//! Schemas for them (`gitvault schema`). Services return DTOs, never domain
//! internals; secret values only travel in [`RevealedSecret`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    domain::{audit::AuditKind, collaborator::CollaboratorRole, secret::SecretKind},
    error::CoreError,
};

/// Derives everything a DTO needs and exports its TypeScript type.
macro_rules! dto {
    ($($item:item)*) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
            #[ts(export)]
            $item
        )*
    };
}

// ---- auth ----------------------------------------------------------------

dto! {
    pub struct AccountDto {
        pub login: String,
        #[ts(type = "number")]
        pub id: u64,
        pub name: Option<String>,
        pub avatar_url: String,
    }

    /// An account known to the local GitHub CLI.
    pub struct GhAccountDto {
        pub login: String,
        /// The account GitVault is currently signed in with.
        pub current: bool,
    }

    pub struct DeviceCodeDto {
        pub device_code: String,
        pub user_code: String,
        pub verification_uri: String,
        pub expires_in: u32,
        pub interval: u32,
    }

    #[serde(tag = "status", rename_all = "snake_case")]
    pub enum LoginPollDto {
        Pending,
        SlowDown,
        Authorized { account: AccountDto },
        Expired,
        Denied,
    }
}

// ---- vaults --------------------------------------------------------------

dto! {
    #[serde(rename_all = "snake_case")]
    pub enum VaultState {
        Local,
        RemoteOnly,
    }

    pub struct VaultSummary {
        /// `None` for vaults that exist only on GitHub (not connected yet).
        pub id: Option<String>,
        pub name: String,
        pub description: String,
        /// `owner/name` when the vault has a GitHub repository.
        pub repo: Option<String>,
        pub state: VaultState,
        pub unlocked: bool,
        pub current: bool,
    }

    pub struct VaultList {
        pub vaults: Vec<VaultSummary>,
        /// Set when GitHub could not be reached (local vaults are still listed).
        pub remote_error: Option<String>,
    }

    pub struct CreateVaultInput {
        pub name: String,
        pub description: Option<String>,
        pub repo_name: Option<String>,
        pub create_remote: bool,
        pub master_password: String,
    }

    pub struct CreatedVault {
        pub vault: VaultSummary,
        /// Shown once. The user must store it safely.
        pub recovery_key: String,
        /// Set when the local vault was created but pushing to GitHub failed.
        pub remote_error: Option<String>,
    }

    pub struct DeviceDto {
        pub id: String,
        pub name: String,
        #[ts(type = "number")]
        pub created_at: i64,
        /// True for the device you are using now.
        pub this_device: bool,
    }

    pub struct FolderRuleDto {
        pub path: String,
        /// Empty means every environment.
        pub environments: Vec<String>,
    }

    pub struct TrustedDeviceDto {
        pub device: DeviceDto,
        /// Only set when the key is meant for another machine (servers, CI): shown once.
        pub device_key: Option<String>,
    }

    pub struct UnlockInput {
        pub vault_id: String,
        pub password: Option<String>,
        pub recovery_key: Option<String>,
    }
}

// ---- collaborators -------------------------------------------------------

dto! {
    /// A GitHub user with access to the vault's repository, or invited to it.
    pub struct CollaboratorDto {
        pub login: String,
        pub avatar_url: String,
        pub role: CollaboratorRole,
        /// Invited but has not accepted yet.
        pub pending: bool,
    }

    pub struct InviteCollaboratorInput {
        pub vault_id: String,
        /// A GitHub username (a leading `@` is ignored).
        pub login: String,
        pub role: CollaboratorRole,
    }
}

// ---- secrets -------------------------------------------------------------

dto! {
    pub struct SecretSummary {
        pub id: String,
        pub name: String,
        pub kind: SecretKind,
        pub environment: Option<String>,
        pub tags: Vec<String>,
        pub favorite: bool,
        #[ts(type = "number | null")]
        pub expires_at: Option<i64>,
        /// Negative when already expired.
        #[ts(type = "number | null")]
        pub expires_in_days: Option<i64>,
        #[ts(type = "number")]
        pub created_at: i64,
        #[ts(type = "number")]
        pub updated_at: i64,
        pub version: u32,
    }

    pub struct SecretDetail {
        #[serde(flatten)]
        pub summary: SecretSummary,
        pub notes: String,
        pub remind_before_days: Option<u32>,
    }

    pub struct AddSecretInput {
        pub vault_id: String,
        pub name: String,
        pub kind: SecretKind,
        pub value: String,
        pub environment: Option<String>,
        pub tags: Vec<String>,
        pub notes: Option<String>,
        #[ts(type = "number | null")]
        pub expires_at: Option<i64>,
        pub remind_before_days: Option<u32>,
    }

    /// Replaces all editable fields. `value` is kept when `None`.
    pub struct UpdateSecretInput {
        pub vault_id: String,
        pub secret_id: String,
        pub name: String,
        pub kind: SecretKind,
        pub value: Option<String>,
        pub environment: Option<String>,
        pub tags: Vec<String>,
        pub notes: Option<String>,
        #[ts(type = "number | null")]
        pub expires_at: Option<i64>,
        pub remind_before_days: Option<u32>,
        pub favorite: bool,
    }

    #[serde(rename_all = "snake_case")]
    pub enum SecretSort {
        Updated,
        Name,
        Expiry,
    }

    pub struct ListSecretsQuery {
        pub vault_id: String,
        pub search: Option<String>,
        pub environment: Option<String>,
        pub tag: Option<String>,
        pub kind: Option<SecretKind>,
        pub favorites_only: bool,
        pub expiring_within_days: Option<u32>,
        pub sort: Option<SecretSort>,
    }

    pub struct FacetCount {
        pub name: String,
        pub count: u32,
    }

    pub struct Facets {
        pub total: u32,
        pub favorites: u32,
        pub expiring_soon: u32,
        pub environments: Vec<FacetCount>,
        pub tags: Vec<FacetCount>,
    }

    /// The only DTO that carries a plaintext secret value.
    pub struct RevealedSecret {
        pub value: String,
    }

    pub struct HistoryEntry {
        pub commit: String,
        /// 1 = oldest.
        pub version: u32,
        pub message: String,
        pub author: String,
        #[ts(type = "number")]
        pub at: i64,
    }

    /// Filter used by export / run: only secrets matching all given fields.
    pub struct EnvFilter {
        pub environment: Option<String>,
        pub tag: Option<String>,
    }

    #[serde(rename_all = "snake_case")]
    pub enum ImportStatus {
        New,
        Duplicate,
    }

    pub struct ImportEnvInput {
        pub vault_id: String,
        pub content: String,
        pub environment: Option<String>,
        /// Replace the value of secrets that already exist (otherwise skip them).
        pub overwrite: bool,
    }

    pub struct ImportPreviewEntry {
        pub name: String,
        /// Value with all but the last characters masked.
        pub masked_value: String,
        pub status: ImportStatus,
    }

    pub struct ImportReport {
        pub created: u32,
        pub updated: u32,
        pub skipped: u32,
    }
}

// ---- sync ----------------------------------------------------------------

dto! {
    #[serde(rename_all = "snake_case")]
    pub enum SyncState {
        NoRemote,
        Synced,
        Pending,
        Conflict,
    }

    pub struct SyncStatus {
        pub state: SyncState,
        pub pending: u32,
        pub conflicts: u32,
        #[ts(type = "number | null")]
        pub last_sync: Option<i64>,
    }

    pub struct SyncResult {
        pub pulled: u32,
        pub pushed: u32,
        pub conflicts: u32,
        pub no_remote: bool,
    }

    pub struct ConflictDto {
        pub secret_id: String,
        /// Name of the local version when the vault is unlocked.
        pub name: Option<String>,
    }

    #[serde(rename_all = "snake_case")]
    pub enum ConflictKeep {
        Local,
        Remote,
    }
}

// ---- tools ---------------------------------------------------------------

dto! {
    #[serde(rename_all = "snake_case")]
    pub enum GenerateKind {
        Password,
        Passphrase,
        Pin,
    }

    pub struct GenerateInput {
        pub kind: GenerateKind,
        pub length: u32,
        pub uppercase: bool,
        pub lowercase: bool,
        pub digits: bool,
        pub symbols: bool,
        pub avoid_ambiguous: bool,
        pub words: u32,
        pub separator: String,
        pub capitalize: bool,
    }

    pub struct GeneratedDto {
        pub value: String,
        pub entropy_bits: f64,
        pub strength: String,
    }
}

// ---- audit ---------------------------------------------------------------

dto! {
    pub struct AuditQuery {
        pub vault_id: Option<String>,
        pub kind: Option<AuditKind>,
        pub search: Option<String>,
        pub limit: Option<u32>,
    }
}

// ---- errors ----------------------------------------------------------------

dto! {
    /// The error shape sent to the UI (and printed by the CLI in `--json` mode).
    pub struct ApiErrorDto {
        /// Stable code such as `E_LOCKED`.
        pub code: String,
        pub message: String,
        pub hint: Option<String>,
    }
}

impl From<&CoreError> for ApiErrorDto {
    fn from(error: &CoreError) -> Self {
        Self {
            code: error.code().as_str().to_owned(),
            message: error.to_string(),
            hint: error.hint().map(str::to_owned),
        }
    }
}
