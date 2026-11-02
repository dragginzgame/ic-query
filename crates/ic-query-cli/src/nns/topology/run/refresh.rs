use crate::cli::common::current_unix_secs;
use crate::nns::{
    NnsCommandError, topology::options::topology_refresh_request, write_text_or_json,
};
use crate::storage::cache_root;
use clap::ArgMatches;
use ic_query::nns::topology::refresh_nns_topology_report;
use ic_query_text::nns::topology::nns_topology_refresh_report_text;
pub(super) async fn run_topology_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let (format, request) =
        topology_refresh_request(matches, network, cache_root()?, current_unix_secs()?);
    let report = refresh_nns_topology_report(&request).await?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
