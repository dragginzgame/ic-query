use super::{announce_missing_catalog, cache_request};
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{NnsCommandError, now_unix_secs, subnet::options::CatalogListOptions},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogListRequest, build_subnet_catalog_list_report,
    subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text,
};
pub(super) fn run_catalog_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = CatalogListOptions::from_matches(matches, network);
    let format = options.format;
    let verbose = options.verbose;
    let cache = cache_request(&options.network)?;
    announce_missing_catalog(&cache, &options.source_endpoint);
    let request = SubnetCatalogListRequest::new(
        cache,
        options.source_endpoint,
        now_unix_secs()?,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_filters(options.filters)
    .with_show_ranges(options.show_ranges)
    .with_range_limit(options.range_limit)
    .with_range_offset(options.range_offset);
    let report = build_subnet_catalog_list_report(&request)?;
    write_text_or_json_verbose(
        format,
        &report,
        verbose,
        subnet_catalog_list_report_text,
        subnet_catalog_list_report_verbose_text,
    )
}
