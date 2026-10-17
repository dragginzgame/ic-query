use super::cache_request;
use crate::cli::common::current_unix_secs;
use crate::nns::{NnsCommandError, leaf::NnsLeafRefreshOptions, write_text_or_json};
use clap::ArgMatches;
use ic_query::nns::nns_inventory_refresh_report_text;
use ic_query::nns::{NnsInventoryRefreshRequest, node::refresh_nns_node_report};
pub(super) async fn run_node_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let mut request = NnsInventoryRefreshRequest::new(
        cache_request(&options.network)?,
        options.source_endpoint,
        current_unix_secs()?,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = refresh_nns_node_report(&request).await?;
    write_text_or_json(format, &report, nns_inventory_refresh_report_text)
}
