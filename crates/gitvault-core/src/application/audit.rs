//! Audit log queries.

use std::sync::Arc;

use super::context::Context;
use crate::{domain::audit::AuditEvent, dto::AuditQuery, error::CoreError};

const DEFAULT_LIMIT: usize = 200;
const SCAN_LIMIT: usize = 5000;

pub struct AuditService {
    ctx: Arc<Context>,
}

impl AuditService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    /// Newest events first, filtered by vault, kind and a text search.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn list(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>, CoreError> {
        let limit = query.limit.map_or(DEFAULT_LIMIT, |n| n as usize);
        let needle = query.search.as_deref().map(str::to_lowercase);
        let events = self.ctx.audit.recent(SCAN_LIMIT)?;
        Ok(events
            .into_iter()
            .filter(|e| query.vault_id.as_ref().is_none_or(|v| &e.vault_id == v))
            .filter(|e| query.kind.is_none_or(|k| e.kind == k))
            .filter(|e| needle.as_ref().is_none_or(|n| matches_text(e, n)))
            .take(limit)
            .collect())
    }
}

fn matches_text(event: &AuditEvent, needle: &str) -> bool {
    event
        .secret_name
        .as_deref()
        .is_some_and(|n| n.to_lowercase().contains(needle))
        || event.device.to_lowercase().contains(needle)
        || event
            .detail
            .as_deref()
            .is_some_and(|d| d.to_lowercase().contains(needle))
}
