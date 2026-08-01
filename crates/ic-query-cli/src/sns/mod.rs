//! SNS command-line parsing and dispatch.

mod commands;

pub use commands::SnsCommandError;
pub use commands::{command, run_matches};
