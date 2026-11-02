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
        options::{proposal_request, proposals_request},
    },
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::sns::build_sns_proposal_report;
use ic_query::sns::build_sns_proposals_cache_list_report;
use ic_query::sns::build_sns_proposals_cache_status_report;
use ic_query::sns::build_sns_proposals_report_with_source_and_progress;
use ic_query::sns::refresh_sns_proposals_cache_with_progress;
use ic_query_text::sns::sns_proposal_report_text;
use ic_query_text::sns::sns_proposals_cache_list_report_text;
use ic_query_text::sns::sns_proposals_cache_status_report_text;
use ic_query_text::sns::sns_proposals_report_text;
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
    let (format, request) =
        proposal_request(matches, network, current_unix_secs()?, || Ok(cache_root()?))?;
    let report = build_sns_proposal_report(&request).await?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

async fn run_sns_proposal_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let (format, request) =
        proposals_request(matches, network, current_unix_secs()?, || Ok(cache_root()?))?;
    let mut progress = StderrQueryProgress::new();
    let report = build_sns_proposals_report_with_source_and_progress(
        &request,
        &ic_query::sns::LiveSnsSource::default(),
        &mut progress,
    )
    .await?;
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
