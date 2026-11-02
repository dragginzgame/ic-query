use super::commands::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
use crate::cli::{
    clap::typed_option,
    common::{OutputFormat, SourceOutputOptions},
};
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::node::NnsNodeListFilters;
use ic_query::nns::node::NnsNodeListRequest;

pub(in crate::nns) fn node_list_request(
    matches: &ArgMatches,
    cache: NnsInventoryCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, bool, NnsNodeListRequest) {
    let source = SourceOutputOptions::from_matches(matches);
    let request = NnsNodeListRequest::new(cache, source.source_endpoint, now_unix_secs)
        .with_filters(NnsNodeListFilters {
            subnet: typed_option(matches, SUBNET_FILTER_ARG),
            subnet_kind: typed_option(matches, SUBNET_KIND_FILTER_ARG),
            data_center: typed_option(matches, DATA_CENTER_FILTER_ARG),
            node_provider: typed_option(matches, NODE_PROVIDER_FILTER_ARG),
            node_operator: typed_option(matches, NODE_OPERATOR_FILTER_ARG),
        });
    (source.format, matches.get_flag("verbose"), request)
}
