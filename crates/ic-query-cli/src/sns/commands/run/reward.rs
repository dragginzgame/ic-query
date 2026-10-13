//! Module: sns::commands::run::reward
//!
//! Responsibility: dispatch SNS reward evidence commands into library builders.
//! Does not own: Clap shape, live Governance calls, or report validation.
//! Boundary: maps parsed checkpoint options into one live report request.

use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    sns::commands::{
        SnsCommandError,
        options::{SnsRewardCheckpointOptions, SnsRewardDiffOptions},
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsRewardCheckpointRequest, build_sns_reward_checkpoint_report,
    build_sns_reward_diff_report_from_paths, sns_reward_checkpoint_report_text,
    sns_reward_diff_report_text,
};

pub(super) async fn run_sns_reward(
    matches: &ArgMatches,
    network: &str,
    network_was_explicit: bool,
) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("checkpoint", matches)) => run_sns_reward_checkpoint(matches, network).await,
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

async fn run_sns_reward_checkpoint(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsRewardCheckpointOptions::from_matches(matches, network);
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let format = lookup.format;
    let request = SnsRewardCheckpointRequest::new(
        lookup.network,
        lookup.source_endpoint,
        now_unix_secs,
        lookup.input,
    )
    .with_max_pages(options.max_pages);
    let report = build_sns_reward_checkpoint_report(&request).await?;
    write_text_or_json(format, &report, sns_reward_checkpoint_report_text)
}
