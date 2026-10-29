use super::cache_request;
use crate::cli::common::current_unix_secs;
use crate::progress::StderrQueryProgress;
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, subnet::options::CatalogInfoOptions},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogInfoRequest,
    build_subnet_catalog_info_report_with_progress, subnet_catalog_info_report_text,
};
pub(super) async fn run_catalog_info(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = CatalogInfoOptions::from_matches(matches, network);
    let format = options.common.format;
    let cache = cache_request(&options.common.network)?;
    let mut request = SubnetCatalogInfoRequest::new(
        cache,
        options.common.source_endpoint,
        options.input,
        current_unix_secs()?,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    if let Some(forced) = options.forced {
        request = request.with_forced(forced);
    }
    let report =
        build_subnet_catalog_info_report_with_progress(&request, &mut StderrQueryProgress::new())
            .await?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}
