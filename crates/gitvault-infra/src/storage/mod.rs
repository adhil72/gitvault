//! File-based adapters: vault clones, private state and the audit log.

mod file_kv_store;
mod fs_vault_storage;
mod io;
mod jsonl_audit_sink;

pub use file_kv_store::FileKeyValueStore;
pub use fs_vault_storage::FsVaultStorage;
pub use jsonl_audit_sink::JsonlAuditSink;
