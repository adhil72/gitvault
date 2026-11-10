use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    System,
}

/// User settings, stored locally (never in the vault repository).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(default)]
pub struct Settings {
    /// Lock the vault after this many idle minutes. 0 disables auto-lock.
    pub auto_lock_minutes: u32,
    /// Clear the clipboard this many seconds after copying a secret. 0 disables.
    pub clear_clipboard_seconds: u32,
    /// Hide a revealed value after this many seconds. 0 disables.
    pub hide_revealed_seconds: u32,
    pub theme: Theme,
    /// Push changes to GitHub automatically after each edit.
    pub auto_sync: bool,
    /// Optional override for the GitHub OAuth app client id.
    pub github_client_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_lock_minutes: 5,
            clear_clipboard_seconds: 30,
            hide_revealed_seconds: 30,
            theme: Theme::Dark,
            auto_sync: true,
            github_client_id: None,
        }
    }
}

impl Settings {
    /// # Errors
    /// Returns a validation error for out-of-range values.
    pub fn validate(&self) -> Result<(), CoreError> {
        let within = |value: u32, max: u32| value <= max;
        if !within(self.auto_lock_minutes, 24 * 60)
            || !within(self.clear_clipboard_seconds, 600)
            || !within(self.hide_revealed_seconds, 600)
        {
            return Err(CoreError::Validation("a setting is out of range".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn defaults_are_valid_and_partial_json_uses_defaults() {
        assert!(Settings::default().validate().is_ok());
        let parsed: Settings = serde_json::from_str(r#"{"auto_sync":false}"#).unwrap();
        assert!(!parsed.auto_sync);
        assert_eq!(parsed.auto_lock_minutes, 5);
    }

    #[test]
    fn rejects_out_of_range() {
        let bad = Settings {
            clear_clipboard_seconds: 9999,
            ..Settings::default()
        };
        assert!(bad.validate().is_err());
    }
}
