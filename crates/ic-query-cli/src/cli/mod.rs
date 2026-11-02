//! Module: cli
//!
//! Responsibility: shared CLI parsing, help, and output utilities.
//! Does not own: NNS/SNS command execution or report construction.
//! Boundary: exposes command-family helpers and clap wrappers used by command modules.

pub mod clap;
pub mod common;
pub mod diagnostics;

#[cfg(test)]
pub use tests::parse_test_args;
#[cfg(test)]
mod tests;
