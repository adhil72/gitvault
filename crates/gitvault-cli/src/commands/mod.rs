//! One module per command group. Each function stays small and only wires
//! input -> service -> output.

pub mod audit;
pub mod auth;
pub mod collaborator;
pub mod device;
pub mod generate;
pub mod load;
pub mod mcp;
pub mod meta;
pub mod organize;
pub mod run;
pub mod secret;
pub mod settings;
pub mod sync;
pub mod transfer;
pub mod trust;
pub mod vault;
