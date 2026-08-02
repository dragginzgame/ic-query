use super::{
    error::SnsCommandError,
    options::{
        SnsListOptions, SnsLookupOptions, SnsMetricsOptions, SnsNeuronsCacheListOptions,
        SnsNeuronsCacheStatusOptions, SnsNeuronsOptions, SnsNeuronsRefreshOptions,
        SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
        SnsProposalsOptions, SnsProposalsRefreshOptions,
    },
    spec::{
        SnsListSortArg, SnsNeuronsSortArg, SnsProposalEligibilityArg, SnsProposalStatusArg,
        SnsProposalTopicArg, SnsProposalsSortArg, sns_canister_command, sns_canister_list_command,
        sns_command, sns_info_command, sns_list_command, sns_metrics_command,
        sns_neuron_cache_command, sns_neuron_cache_list_command, sns_neuron_cache_status_command,
        sns_neuron_command, sns_neuron_list_command, sns_neuron_refresh_command,
        sns_params_command, sns_proposal_cache_list_command, sns_proposal_cache_status_command,
        sns_proposal_command, sns_proposal_info_command, sns_proposal_list_command,
        sns_proposal_refresh_command, sns_swap_command, sns_token_command, sns_upgrade_command,
    },
};
use crate::{cli::clap::parse_matches, cli::common::OutputFormat, test_support::assert_snapshot};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::sns::{DEFAULT_SNS_SOURCE_ENDPOINT, SnsProposalSortDirection};
use ic_query::subnet_catalog::MAINNET_NETWORK;

fn parse_test_matches(command: ClapCommand, args: &[&str]) -> Result<ArgMatches, SnsCommandError> {
    parse_matches(command, args.iter().copied().map(std::ffi::OsString::from))
        .map_err(|error| SnsCommandError::Usage(error.to_string()))
}

fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches, &str) -> Options,
) -> Result<Options, SnsCommandError> {
    let matches = parse_test_matches(command, args)?;
    Ok(from_matches(&matches, MAINNET_NETWORK))
}

fn parse_fallible_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches, &str) -> Result<Options, SnsCommandError>,
) -> Result<Options, SnsCommandError> {
    let matches = parse_test_matches(command, args)?;
    from_matches(&matches, MAINNET_NETWORK)
}

mod canisters;
mod invalid;
mod list;
mod lookup;
mod neurons;
mod proposals;
mod usage;
