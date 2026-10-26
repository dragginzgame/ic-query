use super::cache_request;
use crate::cli::common::current_unix_secs;
use crate::nns::{NnsCommandError, leaf::NnsLeafInfoOptions, write_text_or_json};
use crate::progress::StderrQueryProgress;
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryInfoRequest,
    node::{build_nns_node_info_report_with_progress, nns_node_info_report_text},
};
pub(super) async fn run_node_info(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let cache = cache_request(&options.common.network)?;
    let request = NnsInventoryInfoRequest::new(
        cache,
        options.common.source_endpoint,
        options.input,
        current_unix_secs()?,
    );
    let report =
        build_nns_node_info_report_with_progress(&request, &mut StderrQueryProgress::new()).await?;
    write_text_or_json(options.common.format, &report, nns_node_info_report_text)
}
