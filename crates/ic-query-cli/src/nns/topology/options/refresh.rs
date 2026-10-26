use crate::nns::{leaf::NnsCommonOptions, topology::commands::DRY_RUN_ARG};
use clap::ArgMatches;

///
/// TopologyRefreshOptions
///
/// Parsed options accepted by `icq nns topology refresh`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct TopologyRefreshOptions {
    pub(in crate::nns) common: NnsCommonOptions,

    pub(in crate::nns) dry_run: bool,
}

impl TopologyRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,

            dry_run: matches.get_flag(DRY_RUN_ARG),
        }
    }
}
