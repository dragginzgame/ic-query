use super::{announce_missing_catalog, cache_request};
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, now_unix_secs, subnet::options::CatalogInfoOptions},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogInfoRequest, build_subnet_catalog_info_report,
    subnet_catalog_info_report_text,
};
pub(super) fn run_catalog_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = CatalogInfoOptions::from_matches(matches, network);
    let format = options.format;
    let cache = cache_request(&options.network)?;
    announce_missing_catalog(&cache, &options.source_endpoint);
    let mut request = SubnetCatalogInfoRequest::new(
        cache,
        options.source_endpoint,
        options.input,
        now_unix_secs()?,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    if let Some(forced) = options.forced {
        request = request.with_forced(forced);
    }
    let report = build_subnet_catalog_info_report(&request)?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}
