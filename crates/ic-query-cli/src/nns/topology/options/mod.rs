//! Construct topology refresh requests from CLI inputs and explicit storage/time context.

use crate::{
    cli::common::OutputFormat,
    nns::{leaf::NnsCommonOptions, topology::commands::DRY_RUN_ARG},
};
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryListRequest, topology::NnsTopologyRefreshRequest,
};
use std::path::PathBuf;

pub(in crate::nns) fn topology_refresh_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, NnsTopologyRefreshRequest) {
    let common = NnsCommonOptions::from_matches(matches, network);
    (
        common.format,
        NnsTopologyRefreshRequest::new(NnsInventoryListRequest::new(
            NnsInventoryCacheRequest::new(cache_root, common.network),
            common.source_endpoint,
            now_unix_secs,
        ))
        .with_dry_run(matches.get_flag(DRY_RUN_ARG)),
    )
}
