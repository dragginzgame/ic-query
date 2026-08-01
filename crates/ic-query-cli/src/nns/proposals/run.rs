//! Module: nns::proposals::run
//!
//! Responsibility: run NNS proposal commands.
//! Does not own: clap specs, report construction internals, or text rendering details.
//! Boundary: maps parsed options into report requests and writes text or JSON output.

use super::options::{
    NnsProposalCacheOptions, NnsProposalListOptions, NnsProposalOptions, NnsProposalRefreshOptions,
};
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, command_cache_root, now_unix_secs},
    progress::StderrQueryProgress,
};
use clap::ArgMatches;
use ic_query::nns::proposals::{
    NnsProposalListRequest, NnsProposalRequest, build_nns_proposal_cache_list_report,
    build_nns_proposal_cache_status_report, build_nns_proposal_list_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_report,
    build_nns_proposal_report_from_cache, nns_proposal_cache_list_report_text,
    nns_proposal_cache_status_report_text, nns_proposal_list_report_text,
    nns_proposal_refresh_report_text, nns_proposal_report_text,
    refresh_nns_proposal_cache_with_progress,
};
use ic_query::nns::{NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest};

const PROPOSAL_CACHE_COMMAND: &str = "cache";
const PROPOSAL_CACHE_LIST_COMMAND: &str = "list";
const PROPOSAL_CACHE_STATUS_COMMAND: &str = "status";
const PROPOSAL_INFO_COMMAND: &str = "info";
const PROPOSAL_LIST_COMMAND: &str = "list";
const PROPOSAL_REFRESH_COMMAND: &str = "refresh";

pub(in crate::nns) fn command() -> clap::Command {
    super::commands::nns_proposal_command()
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some((PROPOSAL_CACHE_COMMAND, matches)) => run_nns_proposal_cache(matches, network),
        Some((PROPOSAL_LIST_COMMAND, matches)) => run_nns_proposal_list(matches, network),
        Some((PROPOSAL_INFO_COMMAND, matches)) => run_nns_proposal_info(matches, network),
        Some((PROPOSAL_REFRESH_COMMAND, matches)) => run_nns_proposal_refresh(matches, network),
        _ => unreachable!("clap requires a known NNS proposal subcommand"),
    }
}

fn run_nns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalListOptions::from_matches(matches, network)?;
    run_nns_proposal_list_with_options(options)
}

fn run_nns_proposal_list_with_options(
    options: NnsProposalListOptions,
) -> Result<(), NnsCommandError> {
    let mut request = NnsProposalListRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.limit,
    )
    .with_status(options.status)
    .with_reward_status(options.reward_status)
    .with_topic(options.topic)
    .with_sort(options.sort)
    .with_sort_direction(options.sort_direction)
    .with_verbose(options.verbose);

    if let Some(before_proposal_id) = options.before_proposal_id {
        request = request.with_before_proposal_id(before_proposal_id);
    }
    if let Some(proposer_neuron_id) = options.proposer_neuron_id {
        request = request.with_proposer_neuron_id(proposer_neuron_id);
    }
    if let Some(query) = options.query {
        request = request.with_query(query);
    }

    let report = build_nns_proposal_list_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_proposal_list_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_proposal_list_report_text)
}

fn run_nns_proposal_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalOptions::from_matches(matches, network);
    run_nns_proposal_with_options(options)
}

fn run_nns_proposal_with_options(options: NnsProposalOptions) -> Result<(), NnsCommandError> {
    let request = NnsProposalRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.proposal_id,
    )
    .with_show_ballots(options.show_ballots)
    .with_verbose(options.verbose);
    let report = build_nns_proposal_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_proposal_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_proposal_report_text)
}

fn run_nns_proposal_refresh(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalRefreshOptions::from_matches(matches, network);
    let request = NnsGovernanceRefreshRequest::new(
        command_cache_root()?,
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.page_size,
    )
    .with_max_pages(options.max_pages);
    let mut progress = StderrQueryProgress::new();
    let report = refresh_nns_proposal_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(options.format, &report, nns_proposal_refresh_report_text)
}

fn run_nns_proposal_cache(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some((PROPOSAL_CACHE_LIST_COMMAND, matches)) => {
            run_nns_proposal_cache_list(matches, network)
        }
        Some((PROPOSAL_CACHE_STATUS_COMMAND, matches)) => {
            run_nns_proposal_cache_status(matches, network)
        }
        _ => unreachable!("clap requires a known NNS proposal cache subcommand"),
    }
}

fn run_nns_proposal_cache_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalCacheOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(command_cache_root()?, options.network);
    let report = build_nns_proposal_cache_list_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_cache_list_report_text)
}

fn run_nns_proposal_cache_status(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NnsProposalCacheOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(command_cache_root()?, options.network);
    let report = build_nns_proposal_cache_status_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        nns_proposal_cache_status_report_text,
    )
}
