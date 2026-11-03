use super::cache_request;
use crate::cli::common::current_unix_secs;
use crate::progress::StderrQueryProgress;
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, subnet::options::catalog_info_request},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::build_subnet_catalog_info_report_with_source_and_progress;
use ic_query_text::subnet_catalog::subnet_catalog_info_report_text;
pub(super) async fn run_catalog_info(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let (format, request) =
        catalog_info_request(matches, cache_request(network)?, current_unix_secs()?);
    let report = build_subnet_catalog_info_report_with_source_and_progress(
        &request,
        &ic_query::nns::LiveNnsSource::default(),
        &mut StderrQueryProgress::new(),
    )
    .await?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}
