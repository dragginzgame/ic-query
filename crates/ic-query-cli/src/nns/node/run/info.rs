use super::{announce_missing_node_cache, cache_request};
use crate::nns::{NnsCommandError, leaf::NnsLeafInfoOptions, now_unix_secs, write_text_or_json};
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryInfoRequest,
    node::{build_nns_node_info_report, nns_node_info_report_text},
};
pub(super) fn run_node_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let cache = cache_request(&options.network)?;
    announce_missing_node_cache(&cache, &options.source_endpoint);
    let request = NnsInventoryInfoRequest::new(
        cache,
        options.source_endpoint,
        options.input,
        now_unix_secs()?,
    );
    let report = build_nns_node_info_report(&request)?;
    write_text_or_json(options.format, &report, nns_node_info_report_text)
}
