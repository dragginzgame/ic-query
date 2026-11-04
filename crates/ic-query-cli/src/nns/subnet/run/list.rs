use super::cache_request;
use crate::progress::StderrQueryProgress;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json_verbose},
    nns::{NnsCommandError, subnet::options::catalog_list_request},
};
use clap::ArgMatches;
use ic_query::subnet_catalog::build_subnet_catalog_list_report_with_source_and_progress;
use ic_query_text::subnet_catalog::subnet_catalog_list_report_text;
pub(super) async fn run_catalog_list(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let (format, verbose, request) =
        catalog_list_request(matches, cache_request(network)?, current_unix_secs()?);
    let report = build_subnet_catalog_list_report_with_source_and_progress(
        &request,
        &ic_query::nns::LiveNnsSource::default(),
        &mut StderrQueryProgress::new(),
    )
    .await?;
    write_text_or_json_verbose(format, &report, verbose, subnet_catalog_list_report_text)
}
