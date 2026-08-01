use super::{commands::registry_command, options::RegistryVersionOptions};
use crate::nns::{NnsCommandError, now_unix_secs, write_text_or_json};
use clap::ArgMatches;
use ic_query::nns::registry::{
    NnsRegistryVersionRequest, build_nns_registry_version_report, nns_registry_version_report_text,
};
pub(in crate::nns) fn command() -> clap::Command {
    registry_command()
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("version", matches)) => run_registry_version(matches, network),
        _ => unreachable!("clap requires a known NNS registry subcommand"),
    }
}

fn run_registry_version(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = RegistryVersionOptions::from_matches(matches, network);
    let request =
        NnsRegistryVersionRequest::new(options.network, options.source_endpoint, now_unix_secs()?);
    let report = build_nns_registry_version_report(&request)?;
    write_text_or_json(options.format, &report, nns_registry_version_report_text)
}
