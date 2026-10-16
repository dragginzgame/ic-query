//! Module: nns::proposals::run
//!
//! Responsibility: run NNS proposal commands.
//! Does not own: clap specs, report construction internals, or text rendering details.
//! Boundary: maps parsed options into report requests and writes text or JSON output.

use super::options::{NnsProposalActivityOptions, NnsProposalListOptions, NnsProposalOptions};
use crate::cli::common::{NetworkOutputOptions, current_unix_secs};
use crate::nns::leaf::NnsPagedRefreshOptions;
use crate::storage::cache_root;
use crate::{cli::common::write_text_or_json, nns::NnsCommandError, progress::StderrQueryProgress};
use clap::ArgMatches;
use ic_query::nns::governance::nns_governance_refresh_report_text;
use ic_query::nns::proposals::{
    NnsProposalListRequest, NnsProposalRequest, build_nns_proposal_activity_report_from_cache,
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report, build_nns_proposal_list_report_from_cache,
    build_nns_proposal_report, build_nns_proposal_report_from_cache,
    nns_proposal_activity_report_text, nns_proposal_cache_list_report_text,
    nns_proposal_cache_status_report_text, nns_proposal_list_report_text, nns_proposal_report_text,
    refresh_nns_proposal_cache_with_progress,
};
use ic_query::nns::{
    NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest, governance::NnsGovernanceRequest,
};

const PROPOSAL_CACHE_COMMAND: &str = "cache";
const PROPOSAL_CACHE_LIST_COMMAND: &str = "list";
const PROPOSAL_CACHE_STATUS_COMMAND: &str = "status";
const PROPOSAL_INFO_COMMAND: &str = "info";
const PROPOSAL_LIST_COMMAND: &str = "list";
const PROPOSAL_REFRESH_COMMAND: &str = "refresh";

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("activity", matches)) => run_nns_proposal_activity(matches, network),
        Some((PROPOSAL_CACHE_COMMAND, matches)) => run_nns_proposal_cache(matches, network),
        Some((PROPOSAL_LIST_COMMAND, matches)) => run_nns_proposal_list(matches, network).await,
        Some((PROPOSAL_INFO_COMMAND, matches)) => run_nns_proposal_info(matches, network).await,
        Some((PROPOSAL_REFRESH_COMMAND, matches)) => {
            run_nns_proposal_refresh(matches, network).await
        }
        _ => unreachable!("clap requires a known NNS proposal subcommand"),
    }
}

fn run_nns_proposal_activity(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalActivityOptions::from_matches(matches, network);
    let cache_request = NnsGovernanceCacheRequest::new(cache_root()?, options.network);
    let report = build_nns_proposal_activity_report_from_cache(&cache_request, &options.request)?;
    write_text_or_json(options.format, &report, nns_proposal_activity_report_text)
}

async fn run_nns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalListOptions::from_matches(matches, network)?;
    run_nns_proposal_list_with_options(options).await
}

async fn run_nns_proposal_list_with_options(
    options: NnsProposalListOptions,
) -> Result<(), NnsCommandError> {
    let governance = NnsGovernanceRequest::replica_query_from_unix_secs(
        options.network,
        options.source_endpoint,
        current_unix_secs()?,
        "ic-query",
    );
    let request = NnsProposalListRequest {
        governance,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status,
        reward_status: options.reward_status,
        topic: options.topic,
        proposer_neuron_id: options.proposer_neuron_id,
        query: options.query,
        sort: options.sort,
        sort_direction: options.sort_direction,
        verbose: options.verbose,
    };

    let report = match build_nns_proposal_list_report_from_cache(&request, &cache_root()?)? {
        Some(report) => report,
        None => build_nns_proposal_list_report(&request).await?,
    };
    write_text_or_json(options.format, &report, nns_proposal_list_report_text)
}

async fn run_nns_proposal_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsProposalOptions::from_matches(matches, network);
    run_nns_proposal_with_options(options).await
}

async fn run_nns_proposal_with_options(options: NnsProposalOptions) -> Result<(), NnsCommandError> {
    let governance = NnsGovernanceRequest::replica_query_from_unix_secs(
        options.network,
        options.source_endpoint,
        current_unix_secs()?,
        "ic-query",
    );
    let request = NnsProposalRequest::new(governance, options.proposal_id)
        .with_show_ballots(options.show_ballots)
        .with_verbose(options.verbose);
    let report = match build_nns_proposal_report_from_cache(&request, &cache_root()?)? {
        Some(report) => report,
        None => build_nns_proposal_report(&request).await?,
    };
    write_text_or_json(options.format, &report, nns_proposal_report_text)
}

async fn run_nns_proposal_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NnsPagedRefreshOptions::from_matches(matches, network);
    let request = NnsGovernanceRefreshRequest::new(
        cache_root()?,
        options.network,
        options.source_endpoint,
        current_unix_secs()?,
        options.page_size,
    )
    .with_max_pages(options.max_pages);
    let mut progress = StderrQueryProgress::new();
    let report = refresh_nns_proposal_cache_with_progress(&request, &mut progress).await?;
    write_text_or_json(options.format, &report, nns_governance_refresh_report_text)
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
    let options = NetworkOutputOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(cache_root()?, options.network);
    let report = build_nns_proposal_cache_list_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_cache_list_report_text)
}

fn run_nns_proposal_cache_status(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NetworkOutputOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(cache_root()?, options.network);
    let report = build_nns_proposal_cache_status_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        nns_proposal_cache_status_report_text,
    )
}
