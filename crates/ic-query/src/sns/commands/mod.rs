//! Module: sns::commands
//!
//! Responsibility: expose SNS CLI command dispatch and command errors.
//! Does not own: report construction, live SNS reads, or cache storage.
//! Boundary: keeps SNS clap parsing and runtime wiring separate from reports.

mod error;
mod options;
mod run;
mod spec;

pub use error::SnsCommandError;
pub use run::run;

#[cfg(test)]
mod tests;
