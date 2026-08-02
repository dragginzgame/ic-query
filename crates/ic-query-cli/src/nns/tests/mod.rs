use super::*;
use super::{
    data_center::{DATA_CENTER_SPEC, command as data_center_command},
    governance::{
        NnsGovernanceOptions, governance_command, governance_economics_command,
        governance_maturity_modulation_command, governance_metrics_command,
        governance_reward_event_command,
    },
    leaf::{
        NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions,
        info_command as leaf_info_command, list_command as leaf_list_command,
        refresh_command as leaf_refresh_command,
    },
    neuron::{
        NnsNeuronCacheOptions, NnsNeuronInfoOptions, NnsNeuronListOptions, NnsNeuronRefreshOptions,
        neuron_cache_command, neuron_cache_status_command, neuron_command, neuron_info_command,
        neuron_list_command, neuron_refresh_command,
    },
    node::{NODE_SPEC, node_command, node_list_command, node_list_options_from_matches},
    node_operator::{NODE_OPERATOR_SPEC, command as node_operator_command},
    node_provider::{NODE_PROVIDER_SPEC, command as node_provider_command},
    proposals::{
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL,
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL, NNS_PROPOSAL_SORT_API_LABEL,
        NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_DEADLINE_LABEL,
        NNS_PROPOSAL_SORT_NONE_LABEL, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
        NNS_PROPOSAL_SORT_TALLY_TIME_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
        NNS_PROPOSAL_SORT_VOTING_POWER_LABEL, NNS_PROPOSAL_STATUS_ANY_LABEL,
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_TOPIC_ANY_LABEL,
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL, NnsProposalCacheOptions, NnsProposalListOptions,
        NnsProposalListSort, NnsProposalOptions, NnsProposalRefreshOptions,
        NnsProposalRewardStatusFilter, NnsProposalSortDirection, NnsProposalStatusFilter,
        NnsProposalTopicFilter, nns_proposal_cache_command, nns_proposal_cache_list_command,
        nns_proposal_cache_status_command, nns_proposal_command, nns_proposal_info_command,
        nns_proposal_list_command, nns_proposal_refresh_command,
    },
    registry::{RegistryVersionOptions, registry_command, registry_version_command},
    subnet::{
        CatalogInfoOptions, CatalogListOptions, CatalogRefreshOptions, DEFAULT_RANGE_LIMIT,
        info_command, list_command, refresh_command, subnet_command,
    },
    topology::{
        TopologyCapacityOptions, TopologyCoverageOptions, TopologyGapsOptions,
        TopologyHealthOptions, TopologyProvidersOptions, TopologyReadOptions,
        TopologyRefreshOptions, TopologyRegionsOptions, TopologySummaryOptions,
        TopologyVersionsOptions, topology_capacity_command, topology_command,
        topology_coverage_command, topology_gaps_command, topology_health_command,
        topology_providers_command, topology_refresh_command, topology_regions_command,
        topology_summary_command, topology_versions_command,
    },
};
use crate::cli::clap::parse_matches;
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::nns::{
    data_center::{
        DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    },
    node::{DEFAULT_NNS_NODE_SOURCE_ENDPOINT, DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS},
    node_operator::{
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
    },
    node_provider::{
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT, DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
    },
    registry::DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT,
};
use ic_query::subnet_catalog::{
    DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, GeographicScope, MAINNET_NETWORK, SubnetKind,
    SubnetSpecialization,
};
use std::{ffi::OsString, path::PathBuf};

fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches, &str) -> Options,
) -> Result<Options, NnsCommandError> {
    let matches = parse_test_matches(command, args)?;
    Ok(from_matches(&matches, MAINNET_NETWORK))
}

fn parse_test_matches(command: ClapCommand, args: &[&str]) -> Result<ArgMatches, NnsCommandError> {
    parse_matches(command, args.iter().copied().map(OsString::from))
        .map_err(|error| NnsCommandError::Usage(error.to_string()))
}

mod data_center;
mod governance;
mod neuron;
mod node;
mod node_operator;
mod node_provider;
mod proposals;
mod registry;
mod subnet;
mod topology_help;
mod topology_options;
