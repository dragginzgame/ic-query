use crate::cli::common::current_unix_secs;
use crate::nns::{NnsCommandError, topology::options::TopologyRefreshOptions, write_text_or_json};
use crate::storage::cache_root;
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
    let cache_root = cache_root()?;
    let request =
        NnsTopologyRefreshRequest::new(ic_query::nns::topology::NnsTopologyReadRequest::new(
            cache_root,
            options.network,
            options.source_endpoint,
            current_unix_secs()?,
        ))
        .with_dry_run(options.dry_run);
    let report = refresh_nns_topology_report(&request).await?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
