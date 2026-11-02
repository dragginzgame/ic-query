//! Construct public neuron requests from CLI arguments and the caller's clock.

use crate::{
    cli::{
        clap::{required_typed, typed_option},
        common::OutputFormat,
    },
    nns::leaf::NnsCommonOptions,
};
use clap::ArgMatches;
use ic_query::nns::neuron::NnsNeuronInfoRequest;
use ic_query::nns::neuron::NnsNeuronListRequest;

pub(in crate::nns) fn neuron_list_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsNeuronListRequest) {
    let (format, governance) =
        NnsCommonOptions::from_matches(matches, network).into_governance_request(now_unix_secs);
    (
        format,
        NnsNeuronListRequest {
            governance,
            limit: required_typed(matches, "limit"),
            exclusive_start_neuron_id: typed_option(matches, "start-neuron-id"),
        },
    )
}

pub(in crate::nns) fn neuron_info_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, NnsNeuronInfoRequest) {
    let (format, governance) =
        NnsCommonOptions::from_matches(matches, network).into_governance_request(now_unix_secs);
    (
        format,
        NnsNeuronInfoRequest::new(governance, required_typed(matches, "neuron-id")),
    )
}
