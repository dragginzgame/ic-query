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

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError, options::SnsListOptions, run::common::command_unix_secs, spec::sns_command,
    },
};
use clap::ArgMatches;
use ic_query::sns::{SnsListRequest, build_sns_list_report, sns_list_report_text};
pub fn command() -> clap::Command {
    sns_command()
}

pub fn run_matches(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_sns_list(matches, network),
        Some(("info", matches)) => lookup::run_sns_info(matches, network),
        Some(("metrics", matches)) => lookup::run_sns_metrics(matches, network),
        Some(("token", matches)) => lookup::run_sns_token(matches, network),
        Some(("params", matches)) => lookup::run_sns_params(matches, network),
        Some(("swap", matches)) => lookup::run_sns_swap(matches, network),
        Some(("upgrade", matches)) => lookup::run_sns_upgrade(matches, network),
        Some(("canister", matches)) => canisters::run_sns_canister(matches, network),
        Some(("proposal", matches)) => proposals::run_sns_proposal(matches, network),
        Some(("neuron", matches)) => neurons::run_sns_neuron(matches, network),
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
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let report = build_sns_list_report(&request)?;
    write_text_or_json(format, &report, sns_list_report_text)
}
