//! GitVault core: the single place where business logic lives.
//!
//! Both the desktop app and the CLI call the services exposed through
//! [`services::Services`]. This crate performs no I/O of its own: everything it
//! needs from the outside world is described by traits in [`ports`] and
//! implemented in `gitvault-infra`.
//!
//! - [`domain`]      pure types and rules
//! - [`ports`]       traits the core needs from the outside
//! - [`application`] services (use cases), grouped by area
//! - [`dto`]         types that cross the UI / CLI boundary

pub mod application;
pub mod domain;
pub mod dto;
pub mod error;
pub mod ports;
pub mod services;

pub use error::{CoreError, ErrorCode};
