//! Module: sns::commands::run::reward
//!
//! Responsibility: dispatch SNS reward evidence commands into library builders.
//! Does not own: Clap shape, live Governance calls, or report validation.
//! Boundary: maps parsed checkpoint options into one live report request.

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError,
        options::{SnsRewardCheckpointOptions, SnsRewardDiffOptions},
        run::common::lookup_command_parts,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsRewardCheckpointRequest, build_sns_reward_checkpoint_report,
    build_sns_reward_diff_report_from_paths, sns_reward_checkpoint_report_text,
    sns_reward_diff_report_text,
};

pub(super) fn run_sns_reward(
    matches: &ArgMatches,
    network: &str,
    network_was_explicit: bool,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("checkpoint", matches)) => run_sns_reward_checkpoint(matches, network),
        Some(("diff", matches)) => {
            if network_was_explicit {
                return Err(SnsCommandError::Usage(
                    "--network is not supported by local-only `icq sns reward diff`".to_string(),
                ));
            }
            run_sns_reward_diff(matches)
        }
        _ => unreachable!("clap requires a known SNS reward subcommand"),
    }
}

fn run_sns_reward_diff(matches: &ArgMatches) -> Result<(), SnsCommandError> {
    let options = SnsRewardDiffOptions::from_matches(matches);
    let report = build_sns_reward_diff_report_from_paths(
        &options.before_checkpoint,
        &options.after_checkpoint,
    )?;
    write_text_or_json(options.format, &report, sns_reward_diff_report_text)
}

fn run_sns_reward_checkpoint(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    let options = SnsRewardCheckpointOptions::from_matches(matches, network);
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsRewardCheckpointRequest::new(
        parts.network,
        parts.source_endpoint,
        parts.now_unix_secs,
        parts.input,
    )
    .with_max_pages(options.max_pages);
    let report = build_sns_reward_checkpoint_report(&request)?;
    write_text_or_json(format, &report, sns_reward_checkpoint_report_text)
}
