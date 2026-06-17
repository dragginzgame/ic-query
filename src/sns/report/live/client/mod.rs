//! Module: sns::report::live::client
//!
//! Responsibility: live SNS source adapter root.
//! Does not own: report assembly, cache IO, command parsing, or rendering.
//! Boundary: implements source traits by delegating to live fetch helpers.

mod list;
mod neurons;
mod params;
mod proposals;
mod token;

///
/// LiveSnsSource
///
/// Live mainnet SNS source used by report builders outside tests.
///

pub(in crate::sns::report) struct LiveSnsSource;
