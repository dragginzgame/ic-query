//! Construct catalog requests while keeping text verbosity at the CLI boundary.

use super::commands::{SubnetInfoOptions, SubnetListOptions};
use crate::cli::clap::parsed_options;

use crate::cli::common::OutputFormat;
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
    let options: SubnetListOptions = parsed_options(matches);
    let source = options.source;
    let format = source.format();
    let mut request = SubnetCatalogListRequest::new(
        cache,
        source.source_endpoint,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    request.filters = SubnetCatalogFilters {
        kind: options.kind,
        specialization: options.specialization,
        geographic_scope: options.geo,
    };
    request.show_ranges = options.show_ranges;
    request.range_limit = options.range_limit;
    request.range_offset = options.range_offset;

    (format, options.verbose, request)
}

pub(in crate::nns) fn catalog_info_request(
    matches: &ArgMatches,
    cache: SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> (OutputFormat, SubnetCatalogInfoRequest) {
    let options: SubnetInfoOptions = parsed_options(matches);
    let source = options.source;
    let format = source.format();
    let mut request = SubnetCatalogInfoRequest::new(
        cache,
        source.source_endpoint,
        options.input,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    request.forced = options.forced;
    (format, request)
}
