//! Module: sns::commands::run::proposals
//!
//! Responsibility: run SNS proposal detail, list, refresh, and cache commands.
//! Does not own: proposal cache storage, live governance calls, or rendering.
//! Boundary: maps proposal CLI options into report/cache request DTOs.

use super::cache;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        options::{SnsProposalOptions, SnsProposalsOptions},
    },
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsProposalRequest, SnsProposalsRequest, build_sns_proposal_report,
    build_sns_proposals_cache_list_report, build_sns_proposals_cache_status_report,
    build_sns_proposals_report_with_progress, refresh_sns_proposals_cache_with_progress,
    sns_proposal_report_text, sns_proposals_cache_list_report_text,
    sns_proposals_cache_status_report_text, sns_proposals_report_text,
};
pub(super) async fn run_sns_proposal(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_proposal_list(matches, network).await,
        Some(("info", matches)) => run_sns_proposal_info(matches, network).await,
        Some(("refresh", matches)) => {
            cache::run_snapshot_refresh(matches, network, refresh_sns_proposals_cache_with_progress)
                .await
        }
        Some(("cache", matches)) => run_sns_proposal_cache(matches, network),
        _ => unreachable!("clap requires a known SNS proposal subcommand"),
    }
}

async fn run_sns_proposal_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalOptions::from_matches(matches, network);
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let cache_root = ic_platform::candid::contract::Principal::from_text(&lookup.input)
        .ok()
        .map(|_| cache_root())
        .transpose()?;
    let (format, lookup) = lookup.into_request(now_unix_secs);
    let request = SnsProposalRequest {
        lookup,
        proposal_id: options.proposal_id,
        cache_root,
        verbose: options.verbose,
        show_ballots: options.show_ballots,
    };
    let report = build_sns_proposal_report(&request).await?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

async fn run_sns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsProposalsOptions::from_matches(matches, network)?;
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let cache_root = cache_root()?;
    let (format, lookup) = lookup.into_request(now_unix_secs);
    let request = SnsProposalsRequest {
        lookup,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status.into(),
        topic: options.topic.into(),
        eligibility: options.eligibility.into(),
        proposer_neuron_id: options.proposer_neuron_id,
        query: options.query,
        sort: options.sort.into(),
        sort_direction: options.sort_direction,
        cache_root: Some(cache_root),
        verbose: options.verbose,
    };
    let mut progress = StderrQueryProgress::new();
    let report = build_sns_proposals_report_with_progress(&request, &mut progress).await?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}

fn run_sns_proposal_cache(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => cache::run_cache_list(
            matches,
            network,
            build_sns_proposals_cache_list_report,
            sns_proposals_cache_list_report_text,
        ),
        Some(("status", matches)) => cache::run_cache_status(
            matches,
            network,
            build_sns_proposals_cache_status_report,
            sns_proposals_cache_status_report_text,
        ),
        _ => unreachable!("clap requires a known SNS proposal cache subcommand"),
    }
}
