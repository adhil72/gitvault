//! Adapters that implement the `gitvault-core` ports, plus the single
//! composition root used by both the CLI and the desktop app.
//!
//! - [`storage`]   files on disk (vault clones, private state, audit log)
//! - [`git`]       version control through the `git` command line
//! - [`github`]    GitHub REST API and device flow
//! - [`system`]    clock and ids
//! - [`bootstrap`] wiring everything into `Services`

pub mod bootstrap;
pub mod git;
pub mod github;
pub mod storage;
pub mod system;

pub use bootstrap::{build_services, AppConfig};
