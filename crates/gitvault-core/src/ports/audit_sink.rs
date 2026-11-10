use crate::{domain::audit::AuditEvent, error::CoreError};

/// Append-only local audit log.
pub trait AuditSink: Send + Sync {
    /// # Errors
    /// Fails on a storage error.
    fn append(&self, event: &AuditEvent) -> Result<(), CoreError>;

    /// Newest events first, at most `limit`.
    ///
    /// # Errors
    /// Fails on a storage error.
    fn recent(&self, limit: usize) -> Result<Vec<AuditEvent>, CoreError>;
}
