use super::args::{INFO_INPUT_HELP, INFO_INPUT_VALUE_NAME};
use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID, collection_help};
use clap::Command as ClapCommand;
use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;
use ic_query::subnet_catalog::ResolveAs;

const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq nns subnet info <subnet-prefix>

Refresh stale cache:
  icq nns subnet refresh";

///
/// SubnetInfoOptions
///
/// Typed arguments for the info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::nns) struct SubnetInfoOptions {
    #[arg(value_name = INFO_INPUT_VALUE_NAME, help = INFO_INPUT_HELP)]
    pub(in crate::nns) input: String,
    #[arg(id = "as", long = "as", value_name = "subnet|canister", value_parser = clap::value_parser!(ResolveAs), help = "Force principal interpretation")]
    pub(in crate::nns) forced: Option<ResolveAs>,
    #[command(flatten)]
    pub(in crate::nns) source: crate::cli::common::SourceOutputOptions,
}

pub(in crate::nns) fn info_command() -> ClapCommand {
    with_options::<SubnetInfoOptions>(
        ClapCommand::new("info")
            .about("Resolve a subnet, canister, or subnet prefix to cached subnet info")
            .after_help(collection_help(
                COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID,
                INFO_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
            .help("IC API endpoint used if the subnet catalog cache is missing or invalid")
    })
}
