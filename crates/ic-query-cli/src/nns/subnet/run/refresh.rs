use super::cache_request;
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, leaf::NnsLeafRefreshOptions, now_unix_secs},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{
    CatalogSourceSelection, SubnetCatalogRefreshRequest, refresh_subnet_catalog,
    subnet_catalog_refresh_report_text,
};
pub(super) async fn run_catalog_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let mut request = SubnetCatalogRefreshRequest::new(
        cache_request(&options.network)?,
        CatalogSourceSelection::uncertified_query(options.source_endpoint),
        now_unix_secs()?,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = refresh_subnet_catalog(&request).await?;
    write_text_or_json(format, &report, subnet_catalog_refresh_report_text)
}
