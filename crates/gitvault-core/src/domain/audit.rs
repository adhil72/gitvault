use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AuditKind {
    Created,
    Viewed,
    Copied,
    Updated,
    Rotated,
    Deleted,
    Restored,
    Imported,
    Exported,
    Synced,
    Unlocked,
    Locked,
    DeviceTrusted,
    DeviceRevoked,
    CollaboratorInvited,
    CollaboratorRemoved,
}

/// One line of the local, append-only audit log. It never contains secret values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct AuditEvent {
    #[ts(type = "number")]
    pub at: i64,
    pub kind: AuditKind,
    pub vault_id: String,
    pub secret_id: Option<String>,
    pub secret_name: Option<String>,
    pub device: String,
    pub detail: Option<String>,
}
