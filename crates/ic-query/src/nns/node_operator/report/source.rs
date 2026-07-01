use super::{
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorHostError,
    NnsNodeOperatorListReport, NnsNodeOperatorRow,
};
use crate::{
    ic_registry::{
        MainnetNodeOperatorList, MainnetRegistryFetchRequest, fetch_mainnet_node_operator_list,
    },
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsNodeOperatorSourceRequest
///
/// Source request settings for fetching one complete NNS node-operator snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeOperatorSourceRequest {
    pub network: String,
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsNodeOperatorSourceRequest {
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
/// NnsNodeOperatorSource
///
/// Source contract for fetching complete NNS node-operator list reports.
///

pub trait NnsNodeOperatorSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsNodeOperatorSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError>;
}

///
/// LiveNnsNodeOperatorSource
///
/// Source implementation backed by live NNS registry node-operator calls.
///

pub struct LiveNnsNodeOperatorSource;

impl NnsNodeOperatorSource for LiveNnsNodeOperatorSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsNodeOperatorSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        Ok(node_operator_report_from_list(
            fetch_mainnet_node_operator_list(&fetch_request)?,
        ))
    }
}

pub(super) fn fetch_nns_node_operator_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    super::enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let fetch_request =
        NnsNodeOperatorSourceRequest::new(network, source_endpoint, fetched_at, "ic-query");
    source.fetch_node_operator_list_report(&fetch_request)
}

fn node_operator_report_from_list(list: MainnetNodeOperatorList) -> NnsNodeOperatorListReport {
    let node_operators = list
        .node_operators
        .into_iter()
        .map(|operator| NnsNodeOperatorRow {
            node_operator_principal: operator.principal,
            node_provider_principal: operator.node_provider_principal,
            node_allowance: operator.node_allowance,
            data_center_id: operator.data_center_id,
            node_count: operator.node_count,
        })
        .collect::<Vec<_>>();
    NnsNodeOperatorListReport {
        schema_version: NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_operator_count: node_operators.len(),
        node_operators,
    }
}
