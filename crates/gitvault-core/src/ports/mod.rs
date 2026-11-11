//! Traits describing what the core needs from the outside world.
//!
//! Implemented by `gitvault-infra` (real adapters) and by in-memory fakes in
//! tests. Each port is small and focused (interface segregation).

mod audit_sink;
mod clock;
mod gh_cli;
mod git_host;
mod id_generator;
mod key_value_store;
mod vault_storage;
mod version_control;

pub use audit_sink::AuditSink;
pub use clock::Clock;
pub use gh_cli::{GhCli, GhCliAccount};
pub use git_host::{DeviceCode, DevicePoll, GitHost, RemoteRepo, RepoAccess};
pub use id_generator::IdGenerator;
pub use key_value_store::KeyValueStore;
pub use vault_storage::VaultStorage;
pub use version_control::{
    Author, CommitInfo, ConflictChoice, RepoStatus, SyncReport, VersionControl,
};
