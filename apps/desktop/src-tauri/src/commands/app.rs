use serde::Serialize;

/// Basic information about the running application.
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Returns the application name and version.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "GitVault",
        version: env!("CARGO_PKG_VERSION"),
    }
}
