//! Module: sns::commands::run::neurons::cache
//!
//! Responsibility: run local SNS neuron cache inspection subcommands.
//! Does not own: snapshot discovery, status report construction, or text rendering.
//! Boundary: maps cache CLI options into cache-list and cache-status requests.

use crate::cli::common::NetworkOutputOptions;
use crate::sns::commands::options::SnsCacheStatusOptions;
use crate::{cli::common::write_text_or_json, sns::commands::SnsCommandError, storage::cache_root};
use clap::ArgMatches;
use ic_query::sns::{
    build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
};
pub(super) fn run_sns_neuron_cache(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_neuron_cache_list(matches, network),
        Some(("status", matches)) => run_sns_neuron_cache_status(matches, network),
        _ => unreachable!("clap requires a known SNS neuron cache subcommand"),
    }
}

fn run_sns_neuron_cache_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = NetworkOutputOptions::from_matches(matches, network);
    let (format, request) =
        crate::sns::commands::options::cache_list_request(options, cache_root()?);
    let report = build_sns_neurons_cache_list_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_list_report_text)
}

fn run_sns_neuron_cache_status(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsCacheStatusOptions::from_matches(matches, network);
    let (format, request) = options.into_request(cache_root()?);
    let report = build_sns_neurons_cache_status_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_status_report_text)
}
