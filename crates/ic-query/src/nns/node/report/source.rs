use super::{NNS_NODE_LIST_REPORT_SCHEMA_VERSION, NnsNodeHostError, NnsNodeListReport, NnsNodeRow};
use crate::{
    ic_registry::{MainnetNodeList, MainnetRegistryFetchRequest, fetch_mainnet_node_list},
    subnet_catalog::format_utc_timestamp_secs,
};

pub(super) trait NnsNodeSource {
    fn fetch_nodes(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeList, NnsNodeHostError>;
}

pub(super) struct LiveNnsNodeSource;

impl NnsNodeSource for LiveNnsNodeSource {
    fn fetch_nodes(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeList, NnsNodeHostError> {
        Ok(fetch_mainnet_node_list(request)?)
    }
}

pub(super) fn fetch_nns_node_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    super::enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_nodes(&fetch_request)?;
    Ok(node_report_from_list(list))
}

fn node_report_from_list(list: MainnetNodeList) -> NnsNodeListReport {
    let nodes = list
        .nodes
        .into_iter()
        .map(|node| NnsNodeRow {
            node_principal: node.principal,
            node_operator_principal: node.node_operator_principal,
            node_provider_principal: node.node_provider_principal,
            subnet_principal: node.subnet_principal,
            subnet_kind: node.subnet_kind,
            data_center_id: node.data_center_id,
        })
        .collect::<Vec<_>>();
    NnsNodeListReport {
        schema_version: NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_count: nodes.len(),
        nodes,
    }
}
