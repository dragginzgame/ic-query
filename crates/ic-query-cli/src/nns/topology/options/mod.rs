//! Construct topology refresh requests from CLI inputs and explicit storage/time context.

use super::commands::TopologyRefreshOptions;
use crate::cli::clap::parsed_options;
use crate::{cli::common::OutputFormat, nns::leaf::NnsCommonOptions};
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::NnsInventoryListRequest;
use ic_query::nns::topology::NnsTopologyRefreshRequest;
use std::path::PathBuf;

pub(in crate::nns) fn topology_refresh_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, NnsTopologyRefreshRequest) {
    let options: TopologyRefreshOptions = parsed_options(matches);
    let common = NnsCommonOptions::from_source(options.source, network);
    (common.format(), {
        let mut configured_request = NnsTopologyRefreshRequest::new(NnsInventoryListRequest::new(
            NnsInventoryCacheRequest::new(cache_root, common.network),
            common.source.source_endpoint,
            now_unix_secs,
        ));
        configured_request.dry_run = options.dry_run;
        configured_request
    })
}
