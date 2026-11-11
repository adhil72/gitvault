//! Tauri command handlers, one module per area. Every handler is a thin wrapper
//! over a service in `gitvault-core`, run on a blocking thread.

mod app;
mod auth;
mod secret;
mod sync;
mod tools;
mod vault;

use std::sync::Arc;

use gitvault_core::{dto::ApiErrorDto, services::Services, CoreError};
use tauri::State;

use crate::state::AppState;

/// Runs a blocking service call off the UI thread and maps its error for the UI.
pub async fn blocking<T, F>(state: &State<'_, AppState>, work: F) -> Result<T, ApiErrorDto>
where
    T: Send + 'static,
    F: FnOnce(&Services) -> Result<T, CoreError> + Send + 'static,
{
    let services = Arc::clone(&state.services);
    let joined = tauri::async_runtime::spawn_blocking(move || work(&services)).await;
    match joined {
        Ok(result) => result.map_err(|error| ApiErrorDto::from(&error)),
        Err(error) => Err(ApiErrorDto::from(&CoreError::Internal(error.to_string()))),
    }
}

/// Declares a command: `command!(name(arg: Type, ..) -> Return => |s| expression)`.
macro_rules! command {
    ($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty => |$s:ident| $body:expr) => {
        #[tauri::command]
        pub async fn $name(
            state: tauri::State<'_, crate::state::AppState>,
            $($arg: $ty),*
        ) -> Result<$ret, gitvault_core::dto::ApiErrorDto> {
            crate::commands::blocking(&state, move |$s| $body).await
        }
    };
}
pub(crate) use command;

/// All commands exposed to the frontend (one line per command, so the list is long).
#[allow(clippy::too_many_lines)]
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        app::app_info,
        auth::auth_status,
        auth::auth_login_start,
        auth::auth_login_poll,
        auth::auth_login_token,
        auth::auth_login_gh,
        auth::auth_gh_accounts,
        auth::auth_login_gh_account,
        auth::auth_logout,
        vault::vault_list,
        vault::vault_discover,
        vault::vault_unlock_trusted,
        vault::device_trust,
        vault::device_list,
        vault::device_revoke,
        vault::vault_create,
        vault::vault_connect,
        vault::vault_unlock,
        vault::vault_lock,
        vault::vault_info,
        vault::vault_use,
        vault::vault_forget,
        vault::vault_current,
        vault::vault_change_password,
        vault::vault_regenerate_recovery,
        secret::secret_list,
        secret::secret_facets,
        secret::secret_get,
        secret::secret_reveal,
        secret::secret_mark_copied,
        secret::secret_add,
        secret::secret_update,
        secret::secret_rotate,
        secret::secret_delete,
        secret::secret_history,
        secret::secret_restore,
        secret::import_preview,
        secret::import_env,
        secret::export_env,
        sync::sync_status,
        sync::sync_now,
        sync::sync_conflicts,
        sync::sync_resolve,
        tools::generate,
        tools::audit_list,
        tools::settings_get,
        tools::settings_set,
    ]
}
