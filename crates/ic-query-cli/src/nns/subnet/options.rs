//! Construct catalog requests while keeping text verbosity at the CLI boundary.

use crate::cli::{
    clap::{required_string, required_typed, typed_option},
    common::{OutputFormat, SourceOutputOptions},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::DEFAULT_STALE_AFTER_SECONDS;
use ic_query::subnet_catalog::SubnetCatalogCacheRequest;
use ic_query::subnet_catalog::SubnetCatalogFilters;
use ic_query::subnet_catalog::SubnetCatalogInfoRequest;
use ic_query::subnet_catalog::SubnetCatalogListRequest;

pub(in crate::nns) fn catalog_list_request(
    matches: &ArgMatches,
    cache: SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, bool, SubnetCatalogListRequest) {
    let source = SourceOutputOptions::from_matches(matches);
    let mut request = SubnetCatalogListRequest::new(
        cache,
        source.source_endpoint,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    request.filters = SubnetCatalogFilters {
        kind: typed_option(matches, "kind"),
        specialization: typed_option(matches, "specialization"),
        geographic_scope: typed_option(matches, "geo"),
    };
    request.show_ranges = matches.get_flag("show-ranges");
    request.range_limit = required_typed(matches, "range-limit");
    request.range_offset = required_typed(matches, "range-offset");

    (source.format, matches.get_flag("verbose"), request)
}

pub(in crate::nns) fn catalog_info_request(
    matches: &ArgMatches,
    cache: SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, SubnetCatalogInfoRequest) {
    let source = SourceOutputOptions::from_matches(matches);
    let mut request = SubnetCatalogInfoRequest::new(
        cache,
        source.source_endpoint,
        required_string(matches, "input"),
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    if let Some(forced) = typed_option(matches, "as") {
        request.forced = Some(forced);
    }
    (source.format, request)
}
