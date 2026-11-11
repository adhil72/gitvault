//! Keys used in the private key-value store.

pub const GITHUB_TOKEN: &str = "github.token";
pub const GITHUB_ACCOUNT: &str = "github.account";
pub const SETTINGS: &str = "settings";
pub const CURRENT_VAULT: &str = "current_vault";

pub fn session(vault: &str) -> String {
    format!("session:{vault}")
}

/// This device's own key for a vault it was trusted for.
pub fn device_key(vault: &str) -> String {
    format!("device_key:{vault}")
}

/// Where this device may unlock by itself (folders and environments).
pub fn trust_policy(vault: &str) -> String {
    format!("trust_policy:{vault}")
}

pub fn last_sync(vault: &str) -> String {
    format!("last_sync:{vault}")
}

/// This device's unencrypted index of a vault's secret names (no values, no notes).
pub fn catalog(vault: &str) -> String {
    format!("catalog:{vault}")
}
