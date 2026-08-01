use super::cache_request;
use crate::nns::{NnsCommandError, leaf::NnsLeafRefreshOptions, now_unix_secs, write_text_or_json};
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryRefreshRequest,
    node::{nns_node_refresh_report_text, refresh_nns_node_report},
};
pub(super) fn run_node_refresh(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let mut request = NnsInventoryRefreshRequest::new(
        cache_request(&options.network)?,
        options.source_endpoint,
        now_unix_secs()?,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = refresh_nns_node_report(&request)?;
    write_text_or_json(format, &report, nns_node_refresh_report_text)
}
