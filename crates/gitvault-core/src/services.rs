//! The `Services` object handed to every front door (desktop app, CLI).

use std::sync::Arc;

use crate::application::{
    audit::AuditService, auth::AuthService, context::Context, secrets::SecretService,
    settings::SettingsService, sync::SyncService, tools::ToolsService, vaults::VaultService,
};

/// All services, sharing one [`Context`]. Built once by
/// `gitvault_infra::bootstrap::build_services`.
pub struct Services {
    pub ctx: Arc<Context>,
    pub auth: AuthService,
    pub vaults: VaultService,
    pub secrets: SecretService,
    pub sync: SyncService,
    pub tools: ToolsService,
    pub audit: AuditService,
    pub settings: SettingsService,
}

impl Services {
    #[must_use]
    pub fn new(context: Context) -> Self {
        let ctx = Arc::new(context);
        Self {
            auth: AuthService::new(Arc::clone(&ctx)),
            vaults: VaultService::new(Arc::clone(&ctx)),
            secrets: SecretService::new(Arc::clone(&ctx)),
            sync: SyncService::new(Arc::clone(&ctx)),
            tools: ToolsService,
            audit: AuditService::new(Arc::clone(&ctx)),
            settings: SettingsService::new(Arc::clone(&ctx)),
            ctx,
        }
    }
}
