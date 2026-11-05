//! Module: sns::report::live
//!
//! Responsibility: group live SNS source, fetch, query, conversion, and wire modules.
//! Does not own: command parsing, report assembly, cache IO, or rendering.
//! Boundary: exposes live-source adapters and test-only wire helpers to SNS reports.

mod convert;
mod fetch;
mod query;
mod types;

pub(super) use super::SnsHostError;

///
/// LiveSnsSource
///
/// Live mainnet SNS source used by report builders outside tests.
///

#[derive(Clone, Debug, Default)]
pub struct LiveSnsSource {
    pub(crate) agent: ic_host_tools::agent::EndpointAgent,
}
