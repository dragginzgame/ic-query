use crate::{
    cli::{clap::required_typed, common::OutputFormat},
    nns::{
        leaf::NnsCommonOptions,
        topology::commands::{DRY_RUN_ARG, LOCK_STALE_AFTER_ARG},
    },
};
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;

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
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
}

impl TopologyRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(
            crate::nns::topology::commands::topology_refresh_command(),
            args,
        )?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
