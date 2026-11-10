//! Adapters for operating-system services (time, ids).

mod gh_cli;
mod system_clock;
mod uuid_generator;

pub use gh_cli::GhCliProcess;
pub use system_clock::SystemClock;
pub use uuid_generator::UuidGenerator;
