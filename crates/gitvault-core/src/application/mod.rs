//! Services (use cases) called by both the desktop app and the CLI.
//!
//! One service per area; each operation is a small method. Services share a
//! [`context::Context`] holding the ports.

pub mod audit;
pub mod auth;
pub mod context;
mod devices;
mod keys;
pub mod secrets;
pub mod session;
pub mod settings;
pub mod sync;
pub mod tools;
pub mod vaults;
