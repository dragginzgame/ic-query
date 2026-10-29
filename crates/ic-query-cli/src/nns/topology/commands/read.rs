use crate::{
    cli::common::{COLLECTION_MODE_CACHE_REFRESH_MISSING, collection_help},
    nns::leaf,
};
use ic_query::nns::topology::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT;

const TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if a topology component cache is missing";
const TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the node-operator cache is missing";
const TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the data-center cache is missing";

pub(in crate::nns) fn topology_summary_command() -> clap::Command {
    topology_read_command(
        "summary",
        "Summarize cached mainnet NNS topology reports",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_coverage_command() -> clap::Command {
    topology_read_command(
        "coverage",
        "Show cached mainnet NNS topology join coverage",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_versions_command() -> clap::Command {
    topology_read_command(
        "versions",
        "Show cached mainnet NNS topology component registry versions",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_check_command() -> clap::Command {
    topology_read_command(
        "check",
        "Check cached mainnet NNS topology consistency",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_gaps_command() -> clap::Command {
    topology_read_command(
        "gaps",
        "List cached mainnet NNS topology join gaps",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_capacity_command() -> clap::Command {
    topology_read_command(
        "capacity",
        "Show cached mainnet NNS node-operator capacity",
        TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_regions_command() -> clap::Command {
    topology_read_command(
        "regions",
        "Summarize cached mainnet NNS topology by region",
        TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP,
    )
}

pub(in crate::nns) fn topology_providers_command() -> clap::Command {
    topology_read_command(
        "providers",
        "Summarize cached mainnet NNS topology by node provider",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
    )
}

fn topology_read_command(
    name: &'static str,
    about: &'static str,
    source_help: &'static str,
) -> clap::Command {
    let examples = format!(
        "\
Examples:
  icq nns topology {name}
  icq --network ic nns topology {name} --json
  icq nns topology {name} --source-endpoint https://icp-api.io"
    );
    clap::Command::new(name)
        .bin_name(format!("icq nns topology {name}"))
        .about(about)
        .arg(leaf::json_arg())
        .arg(leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT).help(source_help))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING,
            &examples,
        ))
}
