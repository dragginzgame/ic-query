//! Module: sns::commands::run::neurons
//!
//! Responsibility: run SNS neuron list, refresh, and cache subcommands.
//! Does not own: neuron cache mechanics, live governance calls, or rendering.
//! Boundary: chooses live versus cache-capable request setup from CLI options.

mod cache;
mod refresh;

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError,
        options::{SnsNeuronOptions, SnsNeuronsOptions},
        run::common::{command_cache_root, lookup_command_parts},
        spec::SnsNeuronsSortArg,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsNeuronRequest, SnsNeuronsRequest, SnsNeuronsSort, build_sns_neuron_detail_report,
    build_sns_neurons_report, sns_neuron_detail_report_text, sns_neurons_report_text,
};
use std::path::PathBuf;

pub(super) fn run_sns_neuron(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_neuron_list(matches, network),
        Some(("info", matches)) => run_sns_neuron_info(matches, network),
        Some(("refresh", matches)) => refresh::run_sns_neuron_refresh(matches, network),
        Some(("cache", matches)) => cache::run_sns_neuron_cache(matches, network),
        _ => unreachable!("clap requires a known SNS neuron subcommand"),
    }
}

fn run_sns_neuron_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsNeuronOptions::from_matches(matches, network);
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsNeuronRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        neuron_id: options.neuron_id,
    };
    let report = build_sns_neuron_detail_report(&request)?;
    write_text_or_json(format, &report, sns_neuron_detail_report_text)
}

fn run_sns_neuron_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsNeuronsOptions::from_matches(matches, network)?;
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let cache_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        cache_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn cache_root_for_sort(sort: SnsNeuronsSortArg) -> Result<Option<PathBuf>, SnsCommandError> {
    if SnsNeuronsSort::from(sort).uses_cache() {
        return Ok(Some(command_cache_root()?));
    }
    Ok(None)
}
