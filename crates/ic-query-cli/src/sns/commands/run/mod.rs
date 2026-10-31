//! Module: sns::commands::run
//!
//! Responsibility: dispatch parsed SNS command families into report builders.
//! Does not own: clap command shape, report construction, or text rendering.
//! Boundary: maps command-line options into report requests.

mod cache;
mod lookup;
mod neurons;
mod proposals;
mod reward;

use crate::{
    cli::common::{SourceOutputOptions, current_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
    sns::commands::{SnsCommandError, options::SnsListOptions, spec::sns_command},
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::{
    QueryProgress, QueryProgressEvent,
    sns::{
        SnsCatalogRefreshRequest, SnsListReport, SnsListRequest, build_sns_canister_report,
        build_sns_info_report, build_sns_list_report_from_cache_or_refresh,
        build_sns_params_report, build_sns_swap_report, build_sns_token_report,
        build_sns_upgrade_report, refresh_sns_catalog, sns_canister_report_text,
        sns_catalog_cache_path, sns_catalog_refresh_report_text, sns_info_report_text,
        sns_list_report_text_with_metadata_style, sns_params_report_text, sns_swap_report_text,
        sns_token_report_text, sns_upgrade_report_text,
    },
};
use std::{
    env,
    io::{self, IsTerminal},
};

const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_RESET: &str = "\u{1b}[0m";
pub fn command() -> clap::Command {
    sns_command()
}

pub async fn run_matches(
    matches: &ArgMatches,
    network: &str,
    network_was_explicit: bool,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_list(matches, network).await,
        Some(("refresh", matches)) => run_sns_refresh(matches, network).await,
        Some(("info", matches)) => {
            lookup::run_sns_lookup(
                matches,
                network,
                build_sns_info_report,
                sns_info_report_text,
            )
            .await
        }
        Some(("metrics", matches)) => lookup::run_sns_metrics(matches, network).await,
        Some(("token", matches)) => {
            lookup::run_sns_lookup(
                matches,
                network,
                build_sns_token_report,
                sns_token_report_text,
            )
            .await
        }
        Some(("parameters", matches)) => {
            lookup::run_sns_lookup(
                matches,
                network,
                build_sns_params_report,
                sns_params_report_text,
            )
            .await
        }
        Some(("swap", matches)) => {
            lookup::run_sns_lookup(
                matches,
                network,
                build_sns_swap_report,
                sns_swap_report_text,
            )
            .await
        }
        Some(("upgrade", matches)) => {
            lookup::run_sns_lookup(
                matches,
                network,
                build_sns_upgrade_report,
                sns_upgrade_report_text,
            )
            .await
        }
        Some(("canister", matches)) => match matches.subcommand() {
            Some(("list", matches)) => {
                lookup::run_sns_lookup(
                    matches,
                    network,
                    build_sns_canister_report,
                    sns_canister_report_text,
                )
                .await
            }
            _ => unreachable!("clap requires a known SNS canister subcommand"),
        },
        Some(("proposal", matches)) => proposals::run_sns_proposal(matches, network).await,
        Some(("neuron", matches)) => neurons::run_sns_neuron(matches, network).await,
        Some(("reward", matches)) => {
            reward::run_sns_reward(matches, network, network_was_explicit).await
        }
        _ => unreachable!("clap requires a known SNS subcommand"),
    }
}

async fn run_sns_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsListOptions::from_matches(matches, network);
    let format = options.format;
    let request = SnsListRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        all_lifecycles: options.all_lifecycles,
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let mut progress = StderrQueryProgress::new();
    let report =
        build_sns_list_report_from_cache_or_refresh(&request, &cache_root()?, &mut progress)
            .await?;
    write_text_or_json(format, &report, sns_list_cli_text)
}

fn sns_list_cli_text(report: &SnsListReport) -> String {
    let color = io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none();
    sns_list_report_text_with_metadata_style(report, |status, failed| {
        if color {
            let color = if failed { ANSI_RED } else { ANSI_GREEN };
            format!("{color}{status}{ANSI_RESET}")
        } else {
            status.to_string()
        }
    })
}

async fn run_sns_refresh(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SourceOutputOptions::from_matches(matches);
    let format = options.format;
    let cache_root = cache_root()?;
    let now_unix_secs = current_unix_secs()?;
    let request = SnsCatalogRefreshRequest::new(
        &cache_root,
        network,
        &options.source_endpoint,
        now_unix_secs,
    );
    let mut progress = StderrQueryProgress::new();
    progress.report(QueryProgressEvent::CacheRefresh {
        component: "SNS catalog".to_string(),
        path: sns_catalog_cache_path(&cache_root, &request.cache.network),
        source_endpoint: options.source_endpoint,
    });
    let report = refresh_sns_catalog(&request).await?;
    write_text_or_json(format, &report, sns_catalog_refresh_report_text)
}
