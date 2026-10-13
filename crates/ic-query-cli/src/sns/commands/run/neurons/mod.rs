//! Module: sns::commands::run::neurons
//!
//! Responsibility: run SNS neuron list, refresh, and cache subcommands.
//! Does not own: neuron cache mechanics, live governance calls, or rendering.
//! Boundary: chooses live versus cache-capable request setup from CLI options.

mod cache;
mod refresh;

use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    sns::commands::{
        SnsCommandError,
        options::{SnsNeuronOptions, SnsNeuronsOptions},
        run::common::command_cache_root,
        spec::SnsNeuronsSortArg,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsNeuronRequest, SnsNeuronsRequest, SnsNeuronsSort, build_sns_neuron_detail_report,
    build_sns_neurons_report, sns_neuron_detail_report_text, sns_neurons_report_text,
};
use std::path::PathBuf;

pub(super) async fn run_sns_neuron(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_neuron_list(matches, network).await,
        Some(("info", matches)) => run_sns_neuron_info(matches, network).await,
        Some(("refresh", matches)) => refresh::run_sns_neuron_refresh(matches, network).await,
        Some(("cache", matches)) => cache::run_sns_neuron_cache(matches, network),
        _ => unreachable!("clap requires a known SNS neuron subcommand"),
    }
}

async fn run_sns_neuron_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsNeuronOptions::from_matches(matches, network);
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let format = lookup.format;
    let request = SnsNeuronRequest {
        network: lookup.network,
        source_endpoint: lookup.source_endpoint,
        now_unix_secs,
        input: lookup.input,
        neuron_id: options.neuron_id,
    };
    let report = build_sns_neuron_detail_report(&request).await?;
    write_text_or_json(format, &report, sns_neuron_detail_report_text)
}

async fn run_sns_neuron_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsNeuronsOptions::from_matches(matches, network)?;
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let format = lookup.format;
    let cache_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: lookup.network,
        source_endpoint: lookup.source_endpoint,
        now_unix_secs,
        input: lookup.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        cache_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request).await?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn cache_root_for_sort(sort: SnsNeuronsSortArg) -> Result<Option<PathBuf>, SnsCommandError> {
    if SnsNeuronsSort::from(sort).uses_cache() {
        return Ok(Some(command_cache_root()?));
    }
    Ok(None)
}
