use super::cache_request;
use crate::cli::common::current_unix_secs;
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, leaf::NnsLeafRefreshOptions},
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
    let format = options.common.format;
    let mut request = SubnetCatalogRefreshRequest::new(
        cache_request(&options.common.network)?,
        CatalogSourceSelection::uncertified_query(options.common.source_endpoint),
        current_unix_secs()?,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = refresh_subnet_catalog(&request).await?;
    write_text_or_json(format, &report, subnet_catalog_refresh_report_text)
}
