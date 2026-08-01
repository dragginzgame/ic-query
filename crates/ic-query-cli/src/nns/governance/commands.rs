//! Clap specifications for direct NNS Governance reports.

use crate::{
    cli::{
        clap::{passthrough_subcommand, render_help},
        common::{COLLECTION_MODE_LIVE, collection_help},
    },
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

pub(super) fn governance_command() -> ClapCommand {
    ClapCommand::new("governance")
        .bin_name("icq nns governance")
        .about("Inspect NNS Governance economics, metrics, and rewards")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("economics").about("Show network economics parameters"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("metrics").about("Show cached Governance metrics"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("reward-event").about("Show the latest voting reward event"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("maturity-modulation").about("Show current maturity modulation"),
        ))
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
        .disable_help_flag(true)
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS Governance query"),
        )
        .arg(leaf::network_arg())
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
}

#[cfg(test)]
pub(in crate::nns) fn governance_usage() -> String {
    render_help(governance_command())
}

pub(super) fn governance_usage_for_error() -> String {
    render_help(governance_command())
}

#[cfg(test)]
pub(in crate::nns) fn governance_economics_usage() -> String {
    render_help(governance_economics_command())
}

pub(super) fn governance_economics_usage_for_error() -> String {
    render_help(governance_economics_command())
}

#[cfg(test)]
pub(in crate::nns) fn governance_metrics_usage() -> String {
    render_help(governance_metrics_command())
}

pub(super) fn governance_metrics_usage_for_error() -> String {
    render_help(governance_metrics_command())
}

#[cfg(test)]
pub(in crate::nns) fn governance_reward_event_usage() -> String {
    render_help(governance_reward_event_command())
}

pub(super) fn governance_reward_event_usage_for_error() -> String {
    render_help(governance_reward_event_command())
}

#[cfg(test)]
pub(in crate::nns) fn governance_maturity_modulation_usage() -> String {
    render_help(governance_maturity_modulation_command())
}

pub(super) fn governance_maturity_modulation_usage_for_error() -> String {
    render_help(governance_maturity_modulation_command())
}
