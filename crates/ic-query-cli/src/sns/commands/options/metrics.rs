//! Module: sns::commands::options::metrics
//!
//! Responsibility: parse bounded SNS Governance metrics command options.
//! Does not own: clap command definitions, report requests, or live source calls.
//! Boundary: adds the proposal window to the shared SNS lookup options.

use crate::{cli::clap::required_typed, sns::commands::options::SnsLookupOptions};
use clap::ArgMatches;

///
/// SnsMetricsOptions
///
/// Parsed selector, output, endpoint, and bounded proposal window.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsMetricsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) time_window_seconds: u64,
}

impl SnsMetricsOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            time_window_seconds: required_typed(matches, "window"),
        }
    }
}
