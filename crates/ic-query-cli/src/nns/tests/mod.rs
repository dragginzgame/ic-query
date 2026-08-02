use super::*;
use super::{
    data_center::test_helpers::{
        data_center_info_options, data_center_info_usage, data_center_list_options,
        data_center_list_usage, data_center_refresh_options, data_center_refresh_usage,
        data_center_usage,
    },
    governance::{
        NnsGovernanceOptions, governance_command, governance_economics_command,
        governance_maturity_modulation_command, governance_metrics_command,
        governance_reward_event_command,
    },
    neuron::{
        NnsNeuronCacheOptions, NnsNeuronInfoOptions, NnsNeuronListOptions, NnsNeuronRefreshOptions,
        neuron_cache_command, neuron_cache_status_command, neuron_command, neuron_info_command,
        neuron_list_command, neuron_refresh_command,
    },
    node::{
        node_info_options, node_info_usage, node_list_options, node_list_usage,
        node_refresh_options, node_refresh_usage, node_usage,
    },
    node_operator::test_helpers::{
        node_operator_info_options, node_operator_info_usage, node_operator_list_options,
        node_operator_list_usage, node_operator_refresh_options, node_operator_refresh_usage,
        node_operator_usage,
    },
    node_provider::test_helpers::{
        node_provider_info_options, node_provider_info_usage, node_provider_list_options,
        node_provider_list_usage, node_provider_refresh_options, node_provider_refresh_usage,
        node_provider_usage,
    },
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
    let matches = parse_nns_matches(command, args.iter().copied().map(OsString::from))?;
    Ok(from_matches(&matches, MAINNET_NETWORK))
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
