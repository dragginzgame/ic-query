use super::{
    NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
    NnsSubnetTopologyHostError, NnsSubnetTopologyReport, NnsSubnetTopologyRow,
};
use crate::{
    ic_registry::{MainnetSubnetTopology, fetch_mainnet_subnet_topology},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsSubnetTopologySource
///
/// Source contract for one complete, exact-version Subnet topology report.
///

pub trait NnsSubnetTopologySource {
    /// Fetch and join a complete topology report at one Registry version.
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError>;
}

impl NnsSubnetTopologySource for LiveNnsSource {
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            NnsSubnetTopologyHostError::UnsupportedNetwork { network }
        })?;
        report_from_mainnet_topology(fetch_mainnet_subnet_topology(&fetch_request)?)
    }
}

pub(super) fn source_request(
    network: &str,
    endpoint: &str,
    now_unix_secs: u64,
) -> NnsSourceRequest {
    NnsSourceRequest::new(
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
