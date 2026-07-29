use super::{
    NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
    NnsSubnetTopologyHostError, NnsSubnetTopologyReport, NnsSubnetTopologyRow,
    error::enforce_mainnet_network,
};
use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, MainnetSubnetTopology, fetch_mainnet_subnet_topology,
    },
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsSubnetTopologySourceRequest
///
/// Source settings for collecting one exact-version Subnet topology snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsSubnetTopologySourceRequest {
    /// Network to collect.
    pub network: String,
    /// Replica endpoint used for Registry queries.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl NnsSubnetTopologySourceRequest {
    /// Create source settings for one topology collection.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// NnsSubnetTopologySource
///
/// Source contract for one complete, exact-version Subnet topology report.
///

pub trait NnsSubnetTopologySource {
    /// Fetch and join a complete topology report at one Registry version.
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSubnetTopologySourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError>;
}

///
/// LiveNnsSubnetTopologySource
///
/// Live source backed by one exact-version mainnet Registry inventory.
///

pub struct LiveNnsSubnetTopologySource;

impl NnsSubnetTopologySource for LiveNnsSubnetTopologySource {
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSubnetTopologySourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        enforce_mainnet_network(&request.network)?;
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        report_from_mainnet_topology(fetch_mainnet_subnet_topology(&fetch_request)?)
    }
}

pub(super) fn source_request(
    network: &str,
    endpoint: &str,
    now_unix_secs: u64,
) -> NnsSubnetTopologySourceRequest {
    NnsSubnetTopologySourceRequest::new(
        network,
        endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        "ic-query",
    )
}

fn report_from_mainnet_topology(
    topology: MainnetSubnetTopology,
) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    let subnets = topology
        .subnets
        .into_iter()
        .map(|subnet| NnsSubnetTopologyRow {
            subnet_principal: subnet.subnet_principal,
            subnet_kind: subnet.subnet_kind,
            node_count: subnet.node_count,
            node_providers: subnet
                .node_providers
                .into_iter()
                .map(|provider| NnsSubnetNodeProviderRow {
                    node_provider_principal: provider.node_provider_principal,
                    node_count: provider.node_count,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let report = NnsSubnetTopologyReport {
        schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
        network: topology.network,
        registry_canister_id: topology.registry_canister_id,
        registry_version: topology.registry_version,
        fetched_at: topology.fetched_at,
        source_endpoint: topology.source_endpoint,
        fetched_by: topology.fetched_by,
        subnet_count: subnets.len(),
        node_count: subnets
            .iter()
            .map(|subnet| u64::from(subnet.node_count))
            .sum(),
        subnets,
    };
    report.validate()?;
    Ok(report)
}
