use crate::{
    cli::common::OutputFormat,
    nns::{leaf::NnsCommonOptions, topology::commands::DRY_RUN_ARG},
};
use clap::ArgMatches;

///
/// TopologyRefreshOptions
///
/// Parsed options accepted by `icq nns topology refresh`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct TopologyRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,

    pub(in crate::nns) dry_run: bool,
}

impl TopologyRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,

            dry_run: matches.get_flag(DRY_RUN_ARG),
        }
    }
}
