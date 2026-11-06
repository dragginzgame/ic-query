use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_LIVE, collection_help};
use clap::Command as ClapCommand;
use ic_query::nns::registry::DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT;

const REGISTRY_VERSION_HELP_AFTER: &str = "\
Examples:
  icq nns registry version
  icq --network ic nns registry version --json
  icq nns registry version --source-endpoint https://icp-api.io";

pub(in crate::nns) fn registry_command() -> ClapCommand {
    ClapCommand::new("registry")
        .about("Inspect NNS registry metadata")
        .subcommand(registry_version_command())
}

pub(in crate::nns) fn registry_version_command() -> ClapCommand {
    with_options::<crate::cli::common::SourceOutputOptions>(
        ClapCommand::new("version")
            .about("Show the certified latest mainnet NNS registry version")
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                REGISTRY_VERSION_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT)
            .help("IC API endpoint used for the certified NNS registry query")
    })
}
