//! Parse bounded SNS metrics requests at the CLI boundary.

use crate::{
    cli::{clap::required_typed, common::OutputFormat},
    sns::commands::options::SnsLookupOptions,
};
use clap::ArgMatches;
use ic_query::sns::SnsMetricsRequest;

pub(in crate::sns::commands) fn metrics_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, SnsMetricsRequest) {
    let (format, lookup) =
        SnsLookupOptions::from_matches(matches, network).into_request(now_unix_secs);
    (
        format,
        SnsMetricsRequest::new(lookup).with_time_window_seconds(required_typed(matches, "window")),
    )
}
