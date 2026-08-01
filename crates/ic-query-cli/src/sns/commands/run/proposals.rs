//! Module: sns::commands::run::proposals
//!
//! Responsibility: run SNS proposal detail, list, refresh, and cache commands.
//! Does not own: proposal cache storage, live governance calls, or rendering.
//! Boundary: maps proposal CLI options into report/cache request DTOs.

use crate::{
    cli::common::write_text_or_json,
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        options::{
            SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
            SnsProposalsOptions, SnsProposalsRefreshOptions,
        },
        run::common::{cache_command_parts, cached_lookup_command_parts},
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsProposalRequest, SnsProposalsRefreshRequest,
    SnsProposalsRequest, build_sns_proposal_report, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, build_sns_proposals_report_with_progress,
    refresh_sns_proposals_cache_with_progress, sns_proposal_report_text,
    sns_proposals_cache_list_report_text, sns_proposals_cache_status_report_text,
    sns_proposals_refresh_report_text, sns_proposals_report_text,
};
pub(super) fn run_sns_proposal(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_proposal_list(matches, network),
        Some(("info", matches)) => run_sns_proposal_info(matches, network),
        Some(("refresh", matches)) => run_sns_proposal_refresh(matches, network),
        Some(("cache", matches)) => run_sns_proposal_cache(matches, network),
        _ => unreachable!("clap requires a known SNS proposal subcommand"),
    }
}

fn run_sns_proposal_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalOptions::from_matches(matches, network);
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        proposal_id: options.proposal_id,
        cache_root: Some(parts.cache_root),
        verbose: options.verbose,
        show_ballots: options.show_ballots,
    };
    let report = build_sns_proposal_report(&request)?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

fn run_sns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalsOptions::from_matches(matches, network)?;
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalsRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status.into(),
        topic: options.topic.into(),
        eligibility: options.eligibility.into(),
        proposer_neuron_id: options.proposer_neuron_id,
        query: options.query,
        sort: options.sort.into(),
        sort_direction: options.sort_direction,
        cache_root: Some(parts.cache_root),
        verbose: options.verbose,
    };
    let mut progress = StderrQueryProgress::new();
    let report = build_sns_proposals_report_with_progress(&request, &mut progress)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}

fn run_sns_proposal_refresh(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalsRefreshOptions::from_matches(matches, network);
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalsRefreshRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        cache_root: parts.cache_root,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let mut progress = StderrQueryProgress::new();
    let report = refresh_sns_proposals_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(format, &report, sns_proposals_refresh_report_text)
}

fn run_sns_proposal_cache(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_proposal_cache_list(matches, network),
        Some(("status", matches)) => run_sns_proposal_cache_status(matches, network),
        _ => unreachable!("clap requires a known SNS proposal cache subcommand"),
    }
}

fn run_sns_proposal_cache_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalsCacheListOptions::from_matches(matches, network);
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheListRequest {
        network: parts.network,
        cache_root: parts.cache_root,
    };
    let report = build_sns_proposals_cache_list_report(&request)?;
    write_text_or_json(parts.format, &report, sns_proposals_cache_list_report_text)
}

fn run_sns_proposal_cache_status(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsProposalsCacheStatusOptions::from_matches(matches, network);
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheStatusRequest {
        network: parts.network,
        cache_root: parts.cache_root,
        input: options.input,
    };
    let report = build_sns_proposals_cache_status_report(&request)?;
    write_text_or_json(
        parts.format,
        &report,
        sns_proposals_cache_status_report_text,
    )
}
