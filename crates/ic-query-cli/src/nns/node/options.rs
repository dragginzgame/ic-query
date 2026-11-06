use super::commands::NodeFilters;
use crate::cli::{clap::parsed_options, common::OutputFormat};
use crate::nns::leaf::NnsLeafListOptions;
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::node::NnsNodeListFilters;
use ic_query::nns::node::NnsNodeListRequest;

pub(in crate::nns) fn node_list_request(
    matches: &ArgMatches,
    cache: NnsInventoryCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, bool, NnsNodeListRequest) {
    let options: NnsLeafListOptions = parsed_options(matches);
    let source = options.common.source;
    let filters: NodeFilters = parsed_options(matches);
    let format = source.format();
    let mut request = NnsNodeListRequest::new(cache, source.source_endpoint, now_unix_secs);
    request.filters = NnsNodeListFilters {
        subnet: filters.subnet,
        subnet_kind: filters.subnet_kind,
        data_center: filters.data_center,
        node_provider: filters.node_provider,
        node_operator: filters.node_operator,
    };

    (format, options.verbose, request)
}
