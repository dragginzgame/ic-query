//! Parse bounded SNS metrics requests at the CLI boundary.

use crate::{
    cli::{clap::required_typed, common::OutputFormat},
    sns::commands::options::lookup_request,
};
use clap::ArgMatches;
use ic_query::sns::SnsMetricsRequest;

pub(in crate::sns::commands) fn metrics_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, SnsMetricsRequest), crate::sns::commands::SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    Ok((
        format,
        SnsMetricsRequest::new(lookup).with_time_window_seconds(required_typed(matches, "window")),
    ))
}
