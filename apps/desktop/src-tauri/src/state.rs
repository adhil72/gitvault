//! Shared application state managed by Tauri.

use std::sync::Arc;

use gitvault_core::services::Services;

/// Holds the `Services` object (the same one the CLI uses).
pub struct AppState {
    pub services: Arc<Services>,
}

impl AppState {
    #[must_use]
    pub fn new(services: Services) -> Self {
        Self {
            services: Arc::new(services),
        }
    }
}
