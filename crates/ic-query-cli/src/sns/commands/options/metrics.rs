//! Module: sns::commands::options::metrics
//!
//! Responsibility: parse bounded SNS Governance metrics command options.
//! Does not own: clap command definitions, report requests, or live source calls.
//! Boundary: adds the proposal window to the shared SNS lookup options.

#[cfg(test)]
use crate::sns::commands::{
    SnsCommandError, options::common::parse_sns_matches, spec::sns_metrics_command,
};
use crate::{cli::clap::required_typed, sns::commands::options::SnsLookupOptions};
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;

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
    #[cfg(test)]
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_metrics_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }

    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            time_window_seconds: required_typed(matches, "window"),
        }
    }
}
