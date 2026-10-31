//! Module: sns::commands::run::neurons
//!
//! Responsibility: run SNS neuron list, refresh, and cache subcommands.
//! Does not own: neuron cache mechanics, live governance calls, or rendering.
//! Boundary: chooses live versus cache-capable request setup from CLI options.

use super::cache;

use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    sns::commands::{
        SnsCommandError,
        options::{neuron_request, neurons_request},
    },
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::sns::{
    build_sns_neuron_detail_report, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, build_sns_neurons_report,
    refresh_sns_neurons_cache_with_progress, sns_neuron_detail_report_text,
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_report_text,
};

pub(super) async fn run_sns_neuron(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_neuron_list(matches, network).await,
        Some(("info", matches)) => run_sns_neuron_info(matches, network).await,
        Some(("refresh", matches)) => {
            cache::run_snapshot_refresh(matches, network, refresh_sns_neurons_cache_with_progress)
                .await
        }
        Some(("cache", matches)) => run_sns_neuron_cache(matches, network),
        _ => unreachable!("clap requires a known SNS neuron subcommand"),
    }
}

fn run_sns_neuron_cache(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => cache::run_cache_list(
            matches,
            network,
            build_sns_neurons_cache_list_report,
            sns_neurons_cache_list_report_text,
        ),
        Some(("status", matches)) => cache::run_cache_status(
            matches,
            network,
            build_sns_neurons_cache_status_report,
            sns_neurons_cache_status_report_text,
        ),
        _ => unreachable!("clap requires a known SNS neuron cache subcommand"),
    }
}

async fn run_sns_neuron_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let (format, request) = neuron_request(matches, network, current_unix_secs()?);
    let report = build_sns_neuron_detail_report(&request).await?;
    write_text_or_json(format, &report, sns_neuron_detail_report_text)
}

async fn run_sns_neuron_list(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let (format, request) =
        neurons_request(matches, network, current_unix_secs()?, || Ok(cache_root()?))?;
    let report = build_sns_neurons_report(&request).await?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}
