mod help;

use crate::nns::{leaf, topology::report::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT};
use help::{
    TOPOLOGY_CAPACITY_HELP_AFTER, TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    TOPOLOGY_COVERAGE_HELP_AFTER, TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP, TOPOLOGY_GAPS_HELP_AFTER,
    TOPOLOGY_HEALTH_HELP_AFTER, TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP, TOPOLOGY_PROVIDERS_HELP_AFTER,
    TOPOLOGY_REGIONS_HELP_AFTER, TOPOLOGY_SUMMARY_HELP_AFTER, TOPOLOGY_VERSIONS_HELP_AFTER,
};

pub(in crate::nns::topology) fn topology_summary_command() -> clap::Command {
    topology_read_command(
        "summary",
        "Summarize cached mainnet NNS topology reports",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_SUMMARY_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_coverage_command() -> clap::Command {
    topology_read_command(
        "coverage",
        "Show cached mainnet NNS topology join coverage",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_COVERAGE_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_versions_command() -> clap::Command {
    topology_read_command(
        "versions",
        "Show cached mainnet NNS topology component registry versions",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_VERSIONS_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_health_command() -> clap::Command {
    topology_read_command(
        "health",
        "Check cached mainnet NNS topology cache health",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_HEALTH_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_gaps_command() -> clap::Command {
    topology_read_command(
        "gaps",
        "List cached mainnet NNS topology join gaps",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_GAPS_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_capacity_command() -> clap::Command {
    topology_read_command(
        "capacity",
        "Show cached mainnet NNS node-operator capacity",
        TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP,
        TOPOLOGY_CAPACITY_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_regions_command() -> clap::Command {
    topology_read_command(
        "regions",
        "Summarize cached mainnet NNS topology by region",
        TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP,
        TOPOLOGY_REGIONS_HELP_AFTER,
    )
}

pub(in crate::nns::topology) fn topology_providers_command() -> clap::Command {
    topology_read_command(
        "providers",
        "Summarize cached mainnet NNS topology by node provider",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_PROVIDERS_HELP_AFTER,
    )
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
