use crate::{
    cli::clap::{passthrough_subcommand, render_help},
    nns::{leaf, registry::report::DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT},
};
use clap::Command as ClapCommand;

const REGISTRY_VERSION_HELP_AFTER: &str = "\
Examples:
  icq nns registry version
  icq --network ic nns registry version --format json
  icq nns registry version --source-endpoint https://icp-api.io";

pub(in crate::nns::registry) fn registry_command() -> ClapCommand {
    ClapCommand::new("registry")
        .bin_name("icq nns registry")
        .about("Inspect NNS registry metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("version").about("Show the latest mainnet NNS registry version"),
        ))
}

pub(in crate::nns::registry) fn registry_version_command() -> ClapCommand {
    ClapCommand::new("version")
        .bin_name("icq nns registry version")
        .about("Show the latest mainnet NNS registry version")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS registry query"),
        )
        .arg(leaf::network_arg())
        .after_help(REGISTRY_VERSION_HELP_AFTER)
}

#[cfg(test)]
pub(in crate::nns) fn registry_usage() -> String {
    render_help(registry_command())
}

pub(in crate::nns::registry) fn registry_usage_for_error() -> String {
    render_help(registry_command())
}

#[cfg(test)]
pub(in crate::nns) fn registry_version_usage() -> String {
    render_help(registry_version_command())
}

pub(in crate::nns::registry) fn registry_version_usage_for_error() -> String {
    render_help(registry_version_command())
}
