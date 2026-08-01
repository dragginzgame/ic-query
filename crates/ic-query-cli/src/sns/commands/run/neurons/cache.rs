//! Module: sns::commands::run::neurons::cache
//!
//! Responsibility: run local SNS neuron cache inspection subcommands.
//! Does not own: snapshot discovery, status report construction, or text rendering.
//! Boundary: maps cache CLI options into cache-list and cache-status requests.

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError,
        options::{SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions},
        run::common::cache_command_parts,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsCacheListRequest, SnsCacheStatusRequest, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, sns_neurons_cache_list_report_text,
    sns_neurons_cache_status_report_text,
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
    let options = SnsNeuronsCacheListOptions::from_matches(matches, network);
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheListRequest {
        network: parts.network,
        cache_root: parts.cache_root,
    };
    let report = build_sns_neurons_cache_list_report(&request)?;
    write_text_or_json(parts.format, &report, sns_neurons_cache_list_report_text)
}

fn run_sns_neuron_cache_status(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsNeuronsCacheStatusOptions::from_matches(matches, network);
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheStatusRequest {
        network: parts.network,
        cache_root: parts.cache_root,
        input: options.input,
    };
    let report = build_sns_neurons_cache_status_report(&request)?;
    write_text_or_json(parts.format, &report, sns_neurons_cache_status_report_text)
}
