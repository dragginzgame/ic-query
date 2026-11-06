//! Construct public neuron requests from CLI arguments and the caller's clock.

use super::commands::{NeuronInfoOptions, NeuronListOptions};
use crate::cli::clap::parsed_options;

use crate::{cli::common::OutputFormat, nns::leaf::NnsCommonOptions};
use clap::ArgMatches;
use ic_query::nns::neuron::NnsNeuronInfoRequest;
use ic_query::nns::neuron::NnsNeuronListRequest;

pub(in crate::nns) fn neuron_list_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsNeuronListRequest) {
    let options: NeuronListOptions = parsed_options(matches);
    let (format, governance) = NnsCommonOptions::from_source(options.source, network)
        .into_governance_request(now_unix_secs);
    (
        format,
        NnsNeuronListRequest {
            governance,
            limit: options.limit,
            exclusive_start_neuron_id: options.start_neuron_id,
        },
    )
}

pub(in crate::nns) fn neuron_info_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsNeuronInfoRequest) {
    let options: NeuronInfoOptions = parsed_options(matches);
    let (format, governance) = NnsCommonOptions::from_source(options.source, network)
        .into_governance_request(now_unix_secs);
    (
        format,
        NnsNeuronInfoRequest::new(governance, options.neuron_id),
    )
}
