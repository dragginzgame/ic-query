//! Runtime dispatch for public NNS neuron commands.

use super::{
    commands::neuron_command,
    options::{
        NnsNeuronCacheOptions, NnsNeuronInfoOptions, NnsNeuronListOptions, NnsNeuronRefreshOptions,
    },
};
use crate::{
    nns::{NnsCommandError, command_cache_root, now_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
};
use clap::ArgMatches;
use ic_query::nns::neuron::{
    NnsNeuronInfoRequest, NnsNeuronListRequest, build_nns_neuron_cache_status_report,
    build_nns_neuron_info_report, build_nns_neuron_info_report_from_cache,
    build_nns_neuron_list_report, build_nns_neuron_list_report_from_cache,
    nns_neuron_cache_status_report_text, nns_neuron_info_report_text, nns_neuron_list_report_text,
    nns_neuron_refresh_report_text, refresh_nns_neuron_cache_with_progress,
};
use ic_query::nns::{NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest};
pub(in crate::nns) fn command() -> clap::Command {
    neuron_command()
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_list(matches, network),
        Some(("info", matches)) => run_info(matches, network),
        Some(("refresh", matches)) => run_refresh(matches, network),
        Some(("cache", matches)) => run_cache(matches, network),
        _ => unreachable!("clap requires a known NNS neuron subcommand"),
    }
}

fn run_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronListOptions::from_matches(matches, network);
    let mut request = NnsNeuronListRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.limit,
    )
    .with_verbose(options.verbose);
    if let Some(start_neuron_id) = options.start_neuron_id {
        request = request.with_exclusive_start_neuron_id(start_neuron_id);
    }
    let report = build_nns_neuron_list_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_neuron_list_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_neuron_list_report_text)
}

fn run_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronInfoOptions::from_matches(matches, network);
    let request = NnsNeuronInfoRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.neuron_id,
    )
    .with_verbose(options.verbose);
    let report = build_nns_neuron_info_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_neuron_info_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_neuron_info_report_text)
}

fn run_refresh(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronRefreshOptions::from_matches(matches, network);
    let request = NnsGovernanceRefreshRequest::new(
        command_cache_root()?,
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.page_size,
    )
    .with_max_pages(options.max_pages);
    let mut progress = StderrQueryProgress::new();
    let report = refresh_nns_neuron_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(options.format, &report, nns_neuron_refresh_report_text)
}

fn run_cache(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => run_cache_status(matches, network),
        _ => unreachable!("clap requires a known NNS neuron cache subcommand"),
    }
}

fn run_cache_status(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronCacheOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(command_cache_root()?, options.network);
    let report = build_nns_neuron_cache_status_report(&request)?;
    write_text_or_json(options.format, &report, nns_neuron_cache_status_report_text)
}
