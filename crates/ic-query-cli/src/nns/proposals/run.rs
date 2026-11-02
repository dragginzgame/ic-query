//! Module: nns::proposals::run
//!
//! Responsibility: run NNS proposal commands.
//! Does not own: clap specs, report construction internals, or text rendering details.
//! Boundary: maps parsed options into report requests and writes text or JSON output.

use super::options::{proposal_activity_request, proposal_info_request, proposal_list_request};
use crate::cli::common::{NetworkOutputOptions, current_unix_secs};
use crate::storage::cache_root;
use crate::{cli::common::write_text_or_json, nns::NnsCommandError};
use clap::ArgMatches;
use ic_query::nns::LiveNnsSource;
use ic_query::nns::NnsGovernanceCacheRequest;
use ic_query::nns::proposals::build_nns_proposal_activity_report_from_cache;
use ic_query::nns::proposals::build_nns_proposal_cache_list_report;
use ic_query::nns::proposals::build_nns_proposal_cache_status_report;
use ic_query::nns::proposals::build_nns_proposal_list_report_with_cache_and_source;
use ic_query::nns::proposals::build_nns_proposal_report_with_cache_and_source;
use ic_query::nns::proposals::refresh_nns_proposal_cache_with_progress;
use ic_query_text::nns::proposals::nns_proposal_activity_report_text;
use ic_query_text::nns::proposals::nns_proposal_cache_list_report_text;
use ic_query_text::nns::proposals::nns_proposal_list_report_text;
use ic_query_text::nns::proposals::nns_proposal_report_text;

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("activity", matches)) => run_nns_proposal_activity(matches, network),
        Some(("cache", matches)) => run_nns_proposal_cache(matches, network),
        Some(("list", matches)) => run_nns_proposal_list(matches, network).await,
        Some(("info", matches)) => run_nns_proposal_info(matches, network).await,
        Some(("refresh", matches)) => {
            crate::nns::collection::run_refresh(
                matches,
                network,
                refresh_nns_proposal_cache_with_progress,
            )
            .await
        }
        _ => unreachable!("clap requires a known NNS proposal subcommand"),
    }
}

fn run_nns_proposal_activity(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let (format, request) = proposal_activity_request(matches);
    let cache_request = NnsGovernanceCacheRequest::new(cache_root()?, network);
    let report = build_nns_proposal_activity_report_from_cache(&cache_request, &request)?;
    write_text_or_json(format, &report, nns_proposal_activity_report_text)
}

async fn run_nns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let (format, request) = proposal_list_request(matches, network, current_unix_secs()?)?;

    let report = build_nns_proposal_list_report_with_cache_and_source(
        &request,
        &cache_root()?,
        &LiveNnsSource::default(),
    )
    .await?;
    write_text_or_json(format, &report, nns_proposal_list_report_text)
}

async fn run_nns_proposal_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let (format, request) = proposal_info_request(matches, network, current_unix_secs()?);
    let report = build_nns_proposal_report_with_cache_and_source(
        &request,
        &cache_root()?,
        &LiveNnsSource::default(),
    )
    .await?;
    write_text_or_json(format, &report, nns_proposal_report_text)
}

fn run_nns_proposal_cache(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_nns_proposal_cache_list(matches, network),
        Some(("status", matches)) => crate::nns::collection::run_cache_status(
            matches,
            network,
            build_nns_proposal_cache_status_report,
        ),
        _ => unreachable!("clap requires a known NNS proposal cache subcommand"),
    }
}

fn run_nns_proposal_cache_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NetworkOutputOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(cache_root()?, network);
    let report = build_nns_proposal_cache_list_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_cache_list_report_text)
}
