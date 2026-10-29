//! Runtime dispatch for public NNS neuron commands.

use super::options::{NnsNeuronInfoOptions, NnsNeuronListOptions};
use crate::cli::common::{NetworkOutputOptions, current_unix_secs};
use crate::nns::{NnsCommandError, write_text_or_json};
use crate::storage::cache_root;
use clap::ArgMatches;
use ic_query::nns::neuron::{
    NnsNeuronInfoRequest, NnsNeuronListRequest, build_nns_neuron_cache_status_report,
    build_nns_neuron_distribution_report_from_cache,
    build_nns_neuron_info_report_with_cache_and_source,
    build_nns_neuron_list_report_with_cache_and_source, nns_neuron_distribution_report_text,
    nns_neuron_info_report_text, nns_neuron_list_report_text,
    refresh_nns_neuron_cache_with_progress,
};
use ic_query::nns::{LiveNnsSource, NnsGovernanceCacheRequest};

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("distribution", matches)) => run_distribution(matches, network),
        Some(("list", matches)) => run_list(matches, network).await,
        Some(("info", matches)) => run_info(matches, network).await,
        Some(("refresh", matches)) => {
            crate::nns::collection::run_refresh(
                matches,
                network,
                refresh_nns_neuron_cache_with_progress,
            )
            .await
        }
        Some(("cache", matches)) => run_cache(matches, network),
        _ => unreachable!("clap requires a known NNS neuron subcommand"),
    }
}

fn run_distribution(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NetworkOutputOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(cache_root()?, options.network);
    let report = build_nns_neuron_distribution_report_from_cache(&request)?;
    write_text_or_json(options.format, &report, nns_neuron_distribution_report_text)
}

async fn run_list(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronListOptions::from_matches(matches, network);
    let (format, governance) = options.common.into_governance_request(current_unix_secs()?);
    let request = NnsNeuronListRequest {
        governance,
        limit: options.limit,
        exclusive_start_neuron_id: options.start_neuron_id,
        verbose: options.verbose,
    };
    let report = build_nns_neuron_list_report_with_cache_and_source(
        &request,
        &cache_root()?,
        &LiveNnsSource::default(),
    )
    .await?;
    write_text_or_json(format, &report, nns_neuron_list_report_text)
}

async fn run_info(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    let options = NnsNeuronInfoOptions::from_matches(matches, network);
    let (format, governance) = options.common.into_governance_request(current_unix_secs()?);
    let request =
        NnsNeuronInfoRequest::new(governance, options.neuron_id).with_verbose(options.verbose);
    let report = build_nns_neuron_info_report_with_cache_and_source(
        &request,
        &cache_root()?,
        &LiveNnsSource::default(),
    )
    .await?;
    write_text_or_json(format, &report, nns_neuron_info_report_text)
}

fn run_cache(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => crate::nns::collection::run_cache_status(
            matches,
            network,
            build_nns_neuron_cache_status_report,
        ),
        _ => unreachable!("clap requires a known NNS neuron cache subcommand"),
    }
}
