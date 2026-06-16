use crate::{
    cli::{
        clap::{parse_matches_or_usage, required_typed},
        common::OutputFormat,
    },
    nns::{
        NnsCommandError,
        leaf::NnsCommonOptions,
        topology::commands::{
            DRY_RUN_ARG, LOCK_STALE_AFTER_ARG, topology_refresh_command, topology_refresh_usage,
        },
    },
};
use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct TopologyRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
}

impl TopologyRefreshOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(topology_refresh_command(), args, topology_refresh_usage)
                .map_err(NnsCommandError::Usage)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
        })
    }
}
