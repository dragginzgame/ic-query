use crate::cli::common::current_unix_secs;
use crate::nns::leaf::NnsCommonOptions;
use crate::nns::{NnsCommandError, write_text_or_json};
use clap::ArgMatches;
use ic_query::nns::registry::NnsRegistryVersionRequest;
use ic_query::nns::registry::build_nns_registry_version_report;
use ic_query_text::nns::registry::nns_registry_version_report_text;
pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("version", matches)) => {
            let options = NnsCommonOptions::from_matches(matches, network);
            let format = options.format();
            let request = NnsRegistryVersionRequest::new(
                options.network,
                options.source.source_endpoint,
                current_unix_secs()?,
            );
            let report = build_nns_registry_version_report(&request).await?;
            write_text_or_json(format, &report, nns_registry_version_report_text)
        }
        _ => unreachable!("clap requires a known NNS registry subcommand"),
    }
}
