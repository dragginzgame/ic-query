//! Module: cli
//!
//! Responsibility: shared CLI parsing, help, and output utilities.
//! Does not own: NNS/SNS command execution or report construction.
//! Boundary: exposes command-family helpers and clap wrappers used by command modules.

pub mod clap;
pub mod commands;
pub mod common;
pub mod globals;
pub mod help;

#[cfg(test)]
mod tests;
