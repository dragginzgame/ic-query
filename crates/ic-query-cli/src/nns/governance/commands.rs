//! Clap specifications for direct NNS Governance reports.

use crate::{
    cli::common::{COLLECTION_MODE_LIVE, collection_help},
    nns::leaf,
};
use clap::Command as ClapCommand;
use ic_query::nns::governance::DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT;

const GOVERNANCE_HELP_AFTER: &str = "\
Examples:
  icq nns governance economics
  icq nns governance metrics
  icq nns governance reward-event
  icq nns governance maturity-modulation";

const ECONOMICS_HELP_AFTER: &str = "\
Examples:
  icq nns governance economics
  icq nns governance economics --json";

const METRICS_HELP_AFTER: &str = "\
Examples:
  icq nns governance metrics
  icq nns governance metrics --json";

const REWARD_EVENT_HELP_AFTER: &str = "\
Examples:
  icq nns governance reward-event
  icq nns governance reward-event --json";

const MATURITY_MODULATION_HELP_AFTER: &str = "\
Examples:
  icq nns governance maturity-modulation
  icq nns governance maturity-modulation --json";

pub(in crate::nns) fn governance_command() -> ClapCommand {
    ClapCommand::new("governance")
        .bin_name("icq nns governance")
        .about("Inspect NNS Governance economics, metrics, and rewards")
        .subcommand_required(true)
        .subcommand(governance_economics_command())
        .subcommand(governance_metrics_command())
        .subcommand(governance_reward_event_command())
        .subcommand(governance_maturity_modulation_command())
        .after_help(GOVERNANCE_HELP_AFTER)
}

pub(in crate::nns) fn governance_economics_command() -> ClapCommand {
    report_command(
        "economics",
        "Show NNS Governance network economics parameters",
        ECONOMICS_HELP_AFTER,
    )
}

pub(in crate::nns) fn governance_metrics_command() -> ClapCommand {
    report_command(
        "metrics",
        "Show cached NNS Governance metrics",
        METRICS_HELP_AFTER,
    )
}

pub(in crate::nns) fn governance_reward_event_command() -> ClapCommand {
    report_command(
        "reward-event",
        "Show the latest NNS Governance voting reward event",
        REWARD_EVENT_HELP_AFTER,
    )
}

pub(in crate::nns) fn governance_maturity_modulation_command() -> ClapCommand {
    report_command(
        "maturity-modulation",
        "Show current NNS Governance maturity modulation",
        MATURITY_MODULATION_HELP_AFTER,
    )
}

fn report_command(name: &'static str, about: &'static str, examples: &'static str) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(format!("icq nns governance {name}"))
        .about(about)
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS Governance query"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
}
