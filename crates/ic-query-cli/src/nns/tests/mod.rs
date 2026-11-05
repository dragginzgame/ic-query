use super::*;
use super::{
    data_center::{DATA_CENTER_SPEC, command as data_center_command},
    governance::{
        governance_command, governance_economics_command, governance_maturity_modulation_command,
        governance_metrics_command, governance_reward_event_command,
    },
    leaf::{
        NnsCommonOptions, NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions,
        NnsPagedRefreshOptions, info_command as leaf_info_command,
        list_command as leaf_list_command, refresh_command as leaf_refresh_command,
    },
    neuron::{
        neuron_cache_command, neuron_cache_status_command, neuron_command,
        neuron_distribution_command, neuron_info_command, neuron_info_request, neuron_list_command,
        neuron_list_request, neuron_refresh_command,
    },
    node::{NODE_SPEC, node_command, node_list_command, node_list_request},
    node_operator::{NODE_OPERATOR_SPEC, command as node_operator_command},
    node_provider::{NODE_PROVIDER_SPEC, command as node_provider_command},
    proposals::{
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NnsProposalListSort, NnsProposalRewardStatusFilter,
        NnsProposalStatusFilter, NnsProposalTopicFilter, nns_proposal_activity_command,
        nns_proposal_cache_command, nns_proposal_cache_status_command, nns_proposal_command,
        nns_proposal_info_command, nns_proposal_list_command, nns_proposal_refresh_command,
        proposal_activity_request, proposal_info_request, proposal_list_request,
    },
    registry::{registry_command, registry_version_command},
    subnet::{
        catalog_info_request, catalog_list_request, info_command, list_command, refresh_command,
        subnet_command,
    },
    topology::{
        topology_capacity_command, topology_check_command, topology_command,
        topology_coverage_command, topology_gaps_command, topology_providers_command,
        topology_refresh_command, topology_refresh_request, topology_regions_command,
        topology_summary_command, topology_versions_command,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT;
use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
use ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT;
use ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT;
use ic_query::nns::registry::DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT;
use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;
use ic_query::subnet_catalog::GeographicScope;
use ic_query::subnet_catalog::MAINNET_NETWORK;
use ic_query::subnet_catalog::SubnetKind;
use ic_query::subnet_catalog::SubnetSpecialization;
use std::path::PathBuf;

fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches, &str) -> Options,
) -> Result<Options, NnsCommandError> {
    let matches = parse_test_matches(command, args)?;
    Ok(from_matches(&matches, MAINNET_NETWORK))
}

fn parse_test_matches(command: ClapCommand, args: &[&str]) -> Result<ArgMatches, NnsCommandError> {
    crate::cli::parse_test_args(command, args)
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

const TEST_NOW_UNIX_SECS: u64 = 1_780_531_200;

fn test_node_list_request(
    matches: &ArgMatches,
    network: &str,
) -> (OutputFormat, bool, ic_query::nns::node::NnsNodeListRequest) {
    node_list_request(
        matches,
        ic_query::nns::NnsInventoryCacheRequest::new("/cache", network),
        TEST_NOW_UNIX_SECS,
    )
}

fn test_catalog_list_request(
    matches: &ArgMatches,
    network: &str,
) -> (
    OutputFormat,
    bool,
    ic_query::subnet_catalog::SubnetCatalogListRequest,
) {
    catalog_list_request(
        matches,
        ic_query::subnet_catalog::SubnetCatalogCacheRequest::new("/cache", network),
        TEST_NOW_UNIX_SECS,
    )
}

fn test_catalog_info_request(
    matches: &ArgMatches,
    network: &str,
) -> (
    OutputFormat,
    ic_query::subnet_catalog::SubnetCatalogInfoRequest,
) {
    catalog_info_request(
        matches,
        ic_query::subnet_catalog::SubnetCatalogCacheRequest::new("/cache", network),
        TEST_NOW_UNIX_SECS,
    )
}

fn replica_endpoint(request: &ic_query::nns::governance::NnsGovernanceRequest) -> &str {
    let ic_query::nns::governance::NnsGovernanceSourceSelection::ReplicaQuery { endpoint, .. } =
        &request.source
    else {
        panic!("CLI parser must select a replica query")
    };
    endpoint
}

fn catalog_endpoint(policy: &ic_query::subnet_catalog::CatalogReadPolicy) -> &str {
    let ic_query::subnet_catalog::CatalogReadPolicy::RefreshMissingOrInvalid {
        source: ic_query::subnet_catalog::CatalogSourceSelection::UncertifiedQuery { endpoint },
    } = policy
    else {
        panic!("CLI parser must select missing-or-invalid recovery from one endpoint")
    };
    endpoint
}
