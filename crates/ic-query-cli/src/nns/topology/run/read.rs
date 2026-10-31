use crate::cli::common::current_unix_secs;
use crate::storage::cache_root;
use crate::{
    nns::write_text_or_json,
    nns::{NnsCommandError, leaf::NnsCommonOptions},
};
use clap::ArgMatches;
use ic_query::nns::NnsInventoryListRequest;
use ic_query::nns::topology::NnsTopologyHostError;
use serde::Serialize;

pub(super) async fn run_topology_read<Report>(
    matches: &ArgMatches,
    network: &str,
    build_report: impl AsyncFn(&NnsInventoryListRequest) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Report: Serialize,
{
    let options = NnsCommonOptions::from_matches(matches, network);
    let format = options.format;
    let cache_root = cache_root()?;
    let request = NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(cache_root, options.network),
        options.source_endpoint,
        current_unix_secs()?,
    );
    let report = build_report(&request).await?;
    write_text_or_json(format, &report, render_text)
}
