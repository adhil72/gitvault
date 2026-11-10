use serde::{Deserialize, Serialize};

use super::{SecretKind, SecretValue};

/// Everything about a secret. The whole struct is encrypted into one file, so
/// names, groups and notes are never visible in the GitHub repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretPayload {
    pub name: String,
    pub kind: SecretKind,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub value: SecretValue,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub remind_before_days: Option<u32>,
    #[serde(default)]
    pub favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u32,
}
