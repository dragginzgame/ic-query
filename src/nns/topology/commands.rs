use super::report::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT;
use crate::{
    cli::clap::{flag_arg, passthrough_subcommand, render_help},
    nns::leaf,
};

pub(super) const DRY_RUN_ARG: &str = "dry-run";
pub(super) const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";

const TOPOLOGY_SUMMARY_HELP_AFTER: &str = "\
Examples:
  icq nns topology summary
  icq --network ic nns topology summary --format json
  icq nns topology summary --source-endpoint https://icp-api.io";
const TOPOLOGY_COVERAGE_HELP_AFTER: &str = "\
Examples:
  icq nns topology coverage
  icq --network ic nns topology coverage --format json
  icq nns topology coverage --source-endpoint https://icp-api.io";
const TOPOLOGY_VERSIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology versions
  icq --network ic nns topology versions --format json
  icq nns topology versions --source-endpoint https://icp-api.io";
const TOPOLOGY_HEALTH_HELP_AFTER: &str = "\
Examples:
  icq nns topology health
  icq --network ic nns topology health --format json
  icq nns topology health --source-endpoint https://icp-api.io";
const TOPOLOGY_GAPS_HELP_AFTER: &str = "\
Examples:
  icq nns topology gaps
  icq --network ic nns topology gaps --format json
  icq nns topology gaps --source-endpoint https://icp-api.io";
const TOPOLOGY_CAPACITY_HELP_AFTER: &str = "\
Examples:
  icq nns topology capacity
  icq --network ic nns topology capacity --format json
  icq nns topology capacity --source-endpoint https://icp-api.io";
const TOPOLOGY_REGIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology regions
  icq --network ic nns topology regions --format json
  icq nns topology regions --source-endpoint https://icp-api.io";
const TOPOLOGY_PROVIDERS_HELP_AFTER: &str = "\
Examples:
  icq nns topology providers
  icq --network ic nns topology providers --format json
  icq nns topology providers --source-endpoint https://icp-api.io";
const TOPOLOGY_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns topology refresh
  icq nns topology refresh --dry-run
  icq --network ic nns topology refresh --format json
  icq nns topology refresh --source-endpoint https://icp-api.io";
const TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if a topology component cache is missing";
const TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the node-operator cache is missing";
const TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the data-center cache is missing";
const TOPOLOGY_REFRESH_SOURCE_HELP: &str =
    "IC API endpoint used for NNS topology component refreshes";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TopologyCommandHelp {
    name: &'static str,
    about: &'static str,
}

const TOPOLOGY_SUBCOMMANDS: &[TopologyCommandHelp] = &[
    TopologyCommandHelp {
        name: "summary",
        about: "Summarize cached mainnet NNS topology reports",
    },
    TopologyCommandHelp {
        name: "coverage",
        about: "Show cached mainnet NNS topology join coverage",
    },
    TopologyCommandHelp {
        name: "versions",
        about: "Show cached mainnet NNS topology component registry versions",
    },
    TopologyCommandHelp {
        name: "health",
        about: "Check cached mainnet NNS topology cache health",
    },
    TopologyCommandHelp {
        name: "gaps",
        about: "List cached mainnet NNS topology join gaps",
    },
    TopologyCommandHelp {
        name: "capacity",
        about: "Show cached mainnet NNS node-operator capacity",
    },
    TopologyCommandHelp {
        name: "regions",
        about: "Summarize cached mainnet NNS topology by region",
    },
    TopologyCommandHelp {
        name: "providers",
        about: "Summarize cached mainnet NNS topology by node provider",
    },
    TopologyCommandHelp {
        name: "refresh",
        about: "Refresh cached mainnet NNS topology component reports",
    },
];

pub(super) fn topology_command() -> clap::Command {
    TOPOLOGY_SUBCOMMANDS.iter().fold(
        clap::Command::new("topology")
            .bin_name("icq nns topology")
            .about("Inspect joined NNS topology metadata")
            .disable_help_flag(true),
        |command, subcommand| {
            command.subcommand(passthrough_subcommand(
                clap::Command::new(subcommand.name).about(subcommand.about),
            ))
        },
    )
}

pub(super) fn topology_summary_command() -> clap::Command {
    topology_read_command(
        "summary",
        "Summarize cached mainnet NNS topology reports",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_SUMMARY_HELP_AFTER,
    )
}

pub(super) fn topology_coverage_command() -> clap::Command {
    topology_read_command(
        "coverage",
        "Show cached mainnet NNS topology join coverage",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_COVERAGE_HELP_AFTER,
    )
}

pub(super) fn topology_versions_command() -> clap::Command {
    topology_read_command(
        "versions",
        "Show cached mainnet NNS topology component registry versions",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_VERSIONS_HELP_AFTER,
    )
}

pub(super) fn topology_health_command() -> clap::Command {
    topology_read_command(
        "health",
        "Check cached mainnet NNS topology cache health",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_HEALTH_HELP_AFTER,
    )
}

pub(super) fn topology_gaps_command() -> clap::Command {
    topology_read_command(
        "gaps",
        "List cached mainnet NNS topology join gaps",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_GAPS_HELP_AFTER,
    )
}

pub(super) fn topology_capacity_command() -> clap::Command {
    topology_read_command(
        "capacity",
        "Show cached mainnet NNS node-operator capacity",
        TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP,
        TOPOLOGY_CAPACITY_HELP_AFTER,
    )
}

pub(super) fn topology_regions_command() -> clap::Command {
    topology_read_command(
        "regions",
        "Summarize cached mainnet NNS topology by region",
        TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP,
        TOPOLOGY_REGIONS_HELP_AFTER,
    )
}

pub(super) fn topology_providers_command() -> clap::Command {
    topology_read_command(
        "providers",
        "Summarize cached mainnet NNS topology by node provider",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_PROVIDERS_HELP_AFTER,
    )
}

pub(super) fn topology_refresh_command() -> clap::Command {
    clap::Command::new("refresh")
        .bin_name("icq nns topology refresh")
        .about("Refresh cached mainnet NNS topology component reports")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT)
                .help(TOPOLOGY_REFRESH_SOURCE_HELP),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg(DRY_RUN_ARG)
                .long(DRY_RUN_ARG)
                .help("Fetch and validate without replacing topology component caches"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_REFRESH_HELP_AFTER)
}

fn topology_read_command(
    name: &'static str,
    about: &'static str,
    source_help: &'static str,
    after_help: &'static str,
) -> clap::Command {
    clap::Command::new(name)
        .bin_name(format!("icq nns topology {name}"))
        .about(about)
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT).help(source_help))
        .arg(leaf::network_arg())
        .after_help(after_help)
}

pub(in crate::nns) fn topology_usage() -> String {
    render_help(topology_command())
}

pub(in crate::nns) fn topology_summary_usage() -> String {
    render_help(topology_summary_command())
}

pub(in crate::nns) fn topology_coverage_usage() -> String {
    render_help(topology_coverage_command())
}

pub(in crate::nns) fn topology_versions_usage() -> String {
    render_help(topology_versions_command())
}

pub(in crate::nns) fn topology_health_usage() -> String {
    render_help(topology_health_command())
}

pub(in crate::nns) fn topology_gaps_usage() -> String {
    render_help(topology_gaps_command())
}

pub(in crate::nns) fn topology_capacity_usage() -> String {
    render_help(topology_capacity_command())
}

pub(in crate::nns) fn topology_regions_usage() -> String {
    render_help(topology_regions_command())
}

pub(in crate::nns) fn topology_providers_usage() -> String {
    render_help(topology_providers_command())
}

pub(in crate::nns) fn topology_refresh_usage() -> String {
    render_help(topology_refresh_command())
}
