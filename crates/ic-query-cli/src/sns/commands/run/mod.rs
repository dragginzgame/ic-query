//! Module: sns::commands::run
//!
//! Responsibility: dispatch parsed SNS command families into report builders.
//! Does not own: clap command shape, report construction, or text rendering.
//! Boundary: maps command-line options into report requests.

mod canisters;
mod common;
mod lookup;
mod neurons;
mod proposals;
mod reward;

use crate::{
    cli::common::write_text_or_json,
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        options::{SnsCatalogRefreshOptions, SnsListOptions},
        run::common::{command_cache_root, command_unix_secs},
        spec::sns_command,
    },
};
use clap::ArgMatches;
use ic_query::{
    QueryProgress, QueryProgressEvent,
    sns::{
        DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS, SnsCatalogRefreshRequest, SnsListReport,
        SnsListRequest, build_sns_list_report_from_cache_or_refresh, refresh_sns_catalog,
        sns_catalog_cache_path, sns_catalog_refresh_report_text, sns_list_report_text,
    },
};
use std::{
    env,
    io::{self, IsTerminal},
};

const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_RESET: &str = "\u{1b}[0m";
pub fn command() -> clap::Command {
    sns_command()
}

pub fn run_matches(
    matches: &ArgMatches,
    network: &str,
    network_was_explicit: bool,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_list(matches, network),
        Some(("refresh", matches)) => run_sns_refresh(matches, network),
        Some(("info", matches)) => lookup::run_sns_info(matches, network),
        Some(("metrics", matches)) => lookup::run_sns_metrics(matches, network),
        Some(("token", matches)) => lookup::run_sns_token(matches, network),
        Some(("parameters", matches)) => lookup::run_sns_parameters(matches, network),
        Some(("swap", matches)) => lookup::run_sns_swap(matches, network),
        Some(("upgrade", matches)) => lookup::run_sns_upgrade(matches, network),
        Some(("canister", matches)) => canisters::run_sns_canister(matches, network),
        Some(("proposal", matches)) => proposals::run_sns_proposal(matches, network),
        Some(("neuron", matches)) => neurons::run_sns_neuron(matches, network),
        Some(("reward", matches)) => reward::run_sns_reward(matches, network, network_was_explicit),
        _ => unreachable!("clap requires a known SNS subcommand"),
    }
}

fn run_sns_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsListOptions::from_matches(matches, network);
    let format = options.format;
    let request = SnsListRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        all_lifecycles: options.all_lifecycles,
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let mut progress = StderrQueryProgress::new();
    let report = build_sns_list_report_from_cache_or_refresh(
        &request,
        &command_cache_root()?,
        &mut progress,
    )?;
    write_text_or_json(format, &report, sns_list_cli_text)
}

fn sns_list_cli_text(report: &SnsListReport) -> String {
    let text = sns_list_report_text(report);
    if io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none() {
        color_sns_metadata_statuses(&text)
    } else {
        text
    }
}

fn color_sns_metadata_statuses(text: &str) -> String {
    text.lines()
        .map(|line| {
            for (status, color) in [
                ("ok", ANSI_GREEN),
                ("no_wasm", ANSI_YELLOW),
                ("error", ANSI_RED),
            ] {
                if let Some(prefix) = line.strip_suffix(status)
                    && prefix.ends_with("   ")
                {
                    return format!("{prefix}{color}{status}{ANSI_RESET}");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_sns_refresh(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsCatalogRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let cache_root = command_cache_root()?;
    let now_unix_secs = command_unix_secs()?;
    let request = SnsCatalogRefreshRequest::new(
        &cache_root,
        options.network,
        &options.source_endpoint,
        now_unix_secs,
        DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS,
    );
    let mut progress = StderrQueryProgress::new();
    progress.report(QueryProgressEvent::CacheRefresh {
        component: "SNS catalog".to_string(),
        path: sns_catalog_cache_path(&cache_root, &request.cache.network),
        source_endpoint: options.source_endpoint,
    });
    let report = refresh_sns_catalog(&request)?;
    write_text_or_json(format, &report, sns_catalog_refresh_report_text)
}

#[cfg(test)]
mod tests {
    use super::color_sns_metadata_statuses;

    #[test]
    fn sns_list_metadata_status_color_preserves_table_alignment() {
        let text = "ID   NAME      METADATA\n--   -------   --------\n 1   Ready     ok\n 2   Missing   no_wasm\n 3   Failed    error";
        let colored = color_sns_metadata_statuses(text);

        assert!(colored.contains("Ready     \u{1b}[32mok\u{1b}[0m"));
        assert!(colored.contains("Missing   \u{1b}[33mno_wasm\u{1b}[0m"));
        assert!(colored.contains("Failed    \u{1b}[31merror\u{1b}[0m"));
        assert!(colored.contains("NAME      METADATA"));
    }
}
