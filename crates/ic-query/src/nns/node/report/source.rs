use super::{NNS_NODE_LIST_REPORT_SCHEMA_VERSION, NnsNodeHostError, NnsNodeListReport, NnsNodeRow};
use crate::{
    ic_registry::{MainnetNodeList, MainnetRegistryFetchRequest, fetch_mainnet_node_list},
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsNodeSourceRequest
///
/// Source request settings for fetching one complete NNS node inventory snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeSourceRequest {
    pub network: String,
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsNodeSourceRequest {
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
/// NnsNodeSource
///
/// Source contract for fetching complete NNS node list reports.
///

pub trait NnsNodeSource {
    fn fetch_node_list_report(
        &self,
        request: &NnsNodeSourceRequest,
    ) -> Result<NnsNodeListReport, NnsNodeHostError>;
}

///
/// LiveNnsNodeSource
///
/// Source implementation backed by live NNS registry node inventory calls.
///

pub struct LiveNnsNodeSource;

impl NnsNodeSource for LiveNnsNodeSource {
    fn fetch_node_list_report(
        &self,
        request: &NnsNodeSourceRequest,
    ) -> Result<NnsNodeListReport, NnsNodeHostError> {
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        Ok(node_report_from_list(fetch_mainnet_node_list(
            &fetch_request,
        )?))
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
    let fetch_request = NnsNodeSourceRequest::new(network, source_endpoint, fetched_at, "ic-query");
    source.fetch_node_list_report(&fetch_request)
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
