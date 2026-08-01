use super::{announce_missing_node_cache, cache_request};
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{NnsCommandError, node::options::node_list_options_from_matches, now_unix_secs},
};
use clap::ArgMatches;
use ic_query::nns::node::{
    NnsNodeListRequest, build_nns_node_list_report, nns_node_list_report_text,
    nns_node_list_report_verbose_text,
};
pub(super) fn run_node_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = node_list_options_from_matches(matches, network);
    let cache = cache_request(&options.network)?;
    announce_missing_node_cache(&cache, &options.source_endpoint);
    let request = NnsNodeListRequest::new(cache, options.source_endpoint, now_unix_secs()?)
        .with_filters(options.filters);
    let report = build_nns_node_list_report(&request)?;
    write_text_or_json_verbose(
        options.format,
        &report,
        options.verbose,
        nns_node_list_report_text,
        nns_node_list_report_verbose_text,
    )
}
