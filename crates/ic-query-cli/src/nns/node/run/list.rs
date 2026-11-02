use super::cache_request;
use crate::progress::StderrQueryProgress;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json_verbose},
    nns::{NnsCommandError, node::options::node_list_request},
};
use clap::ArgMatches;
use ic_query::nns::node::{
    build_nns_node_list_report_with_source_and_progress, nns_node_list_report_text,
    nns_node_list_report_verbose_text,
};
pub(super) async fn run_node_list(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let (format, verbose, request) =
        node_list_request(matches, cache_request(network)?, current_unix_secs()?);
    let report = build_nns_node_list_report_with_source_and_progress(
        &request,
        &ic_query::nns::LiveNnsSource::default(),
        &mut StderrQueryProgress::new(),
    )
    .await?;
    write_text_or_json_verbose(
        format,
        &report,
        verbose,
        nns_node_list_report_text,
        nns_node_list_report_verbose_text,
    )
}
