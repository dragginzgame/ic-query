use crate::{
    cli::common::{COLLECTION_MODE_LIVE, collection_help},
    nns::leaf,
};
use clap::Command as ClapCommand;
use ic_query::nns::registry::DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT;

const REGISTRY_VERSION_HELP_AFTER: &str = "\
Examples:
  icq nns registry version
  icq --network ic nns registry version --json
  icq nns registry version --source-endpoint https://icp-api.io";

pub(in crate::nns) fn registry_command() -> ClapCommand {
    ClapCommand::new("registry")
        .bin_name("icq nns registry")
        .about("Inspect NNS registry metadata")
        .subcommand(registry_version_command())
}

pub(in crate::nns) fn registry_version_command() -> ClapCommand {
    ClapCommand::new("version")
        .bin_name("icq nns registry version")
        .about("Show the certified latest mainnet NNS registry version")
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the certified NNS registry query"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            REGISTRY_VERSION_HELP_AFTER,
        ))
}
