//! GitVault desktop shell.
//!
//! This crate only registers Tauri commands and OS integration. Every command
//! is a thin wrapper: call a service on `Services`, map the error. No business
//! logic lives here.

mod commands;
mod state;

use gitvault_infra::{build_services, AppConfig};
use state::AppState;

/// Starts the desktop application.
///
/// # Errors
/// Returns an error when the services or Tauri fail to start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let services = build_services(&AppConfig::from_env("Desktop", false))?;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new(services))
        .invoke_handler(commands::handler())
        .run(tauri::generate_context!())?;
    Ok(())
}
