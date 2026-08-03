//! Module: sns::commands::spec::commands::reward
//!
//! Responsibility: build Clap specs for SNS reward evidence commands.
//! Does not own: checkpoint collection, report construction, or output writing.
//! Boundary: defines the live checkpoint command and its explicit diagnostic page cap.

use crate::{
    cli::{
        clap::value_arg,
        common::{
            COLLECTION_MODE_LIVE, COLLECTION_MODE_LOCAL_ONLY, collection_help, json_arg,
            source_endpoint_arg,
        },
    },
    sns::commands::spec::commands::args::sns_lookup_input_arg,
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;
use std::path::PathBuf;

const SNS_REWARD_CHECKPOINT_HELP_AFTER: &str = "\
Examples:
  icq sns reward checkpoint 1
  icq sns reward checkpoint 23ten-uaaaa-aaaaq-aabia-cai --json
  icq sns reward checkpoint 1 --max-pages 10 --json

The collector brackets strict list_neurons exhaustion with complete Governance
parameters, reward-event, and running-version responses. For N neuron pages it
makes N + 8 client queries including targeted SNS discovery. It does not query
proposal ballots, ledgers, transactions, or individual neuron detail methods.";

const SNS_REWARD_DIFF_HELP_AFTER: &str = "\
Examples:
  icq sns reward diff before.json after.json
  icq sns reward diff before.json after.json --json

Both files are treated as untrusted current-schema checkpoints. The command
recomputes their raw rows and policy findings, performs no live calls, and
reports an allocation only after exact native reward-event reconciliation.";

pub(in crate::sns::commands) fn sns_reward_command() -> ClapCommand {
    ClapCommand::new("reward")
        .bin_name("icq sns reward")
        .about("Collect and compare SNS maturity reward evidence")
        .subcommand_required(true)
        .subcommand(sns_reward_checkpoint_command())
        .subcommand(sns_reward_diff_command())
}

pub(in crate::sns::commands) fn sns_reward_diff_command() -> ClapCommand {
    ClapCommand::new("diff")
        .bin_name("icq sns reward diff")
        .about("Locally reconcile two untrusted SNS reward checkpoints")
        .arg(
            value_arg("before-checkpoint")
                .required(true)
                .value_name("before.json")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            value_arg("after-checkpoint")
                .required(true)
                .value_name("after.json")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_LOCAL_ONLY,
            SNS_REWARD_DIFF_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_reward_checkpoint_command() -> ClapCommand {
    ClapCommand::new("checkpoint")
        .bin_name("icq sns reward checkpoint")
        .about("Collect an API-exhausted observed SNS maturity checkpoint")
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and Governance queries"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Optional diagnostic page cap; reaching it before exhaustion is an error"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            SNS_REWARD_CHECKPOINT_HELP_AFTER,
        ))
}
