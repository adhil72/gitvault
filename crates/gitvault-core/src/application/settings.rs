//! Settings service.

use std::sync::Arc;

use super::{context::Context, keys};
use crate::{domain::settings::Settings, error::CoreError};

pub struct SettingsService {
    ctx: Arc<Context>,
}

impl SettingsService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    #[must_use]
    pub fn get(&self) -> Settings {
        self.ctx.settings()
    }

    /// Validates and stores new settings.
    ///
    /// # Errors
    /// Validation error for out-of-range values.
    pub fn update(&self, settings: &Settings) -> Result<Settings, CoreError> {
        settings.validate()?;
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        self.ctx.kv.set(keys::SETTINGS, &json)?;
        Ok(settings.clone())
    }
}
