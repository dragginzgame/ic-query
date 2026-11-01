//! Construct catalog requests while keeping text verbosity at the CLI boundary.

use crate::cli::{
    clap::{required_string, required_typed, typed_option},
    common::{OutputFormat, SourceOutputOptions},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogCacheRequest, SubnetCatalogFilters,
    SubnetCatalogInfoRequest, SubnetCatalogListRequest,
};

pub(in crate::nns) fn catalog_list_request(
    matches: &ArgMatches,
    cache: SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, bool, SubnetCatalogListRequest) {
    let source = SourceOutputOptions::from_matches(matches);
    let request = SubnetCatalogListRequest::new(
        cache,
        source.source_endpoint,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_filters(SubnetCatalogFilters {
        kind: typed_option(matches, "kind"),
        specialization: typed_option(matches, "specialization"),
        geographic_scope: typed_option(matches, "geo"),
    })
    .with_show_ranges(matches.get_flag("show-ranges"))
    .with_range_limit(required_typed(matches, "range-limit"))
    .with_range_offset(required_typed(matches, "range-offset"));
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
        request = request.with_forced(forced);
    }
    (source.format, request)
}
