use crate::cli::common::current_unix_secs;
use crate::nns::{
    NnsCommandError, command_cache_root, topology::options::TopologyRefreshOptions,
    write_text_or_json,
};
use clap::ArgMatches;
use ic_query::nns::topology::{
    NnsTopologyRefreshRequest, nns_topology_refresh_report_text, refresh_nns_topology_report,
};
pub(super) async fn run_topology_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = TopologyRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let cache_root = command_cache_root()?;
    let request = NnsTopologyRefreshRequest::new(
        cache_root,
        options.network,
        options.source_endpoint,
        current_unix_secs()?,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    let report = refresh_nns_topology_report(&request).await?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
