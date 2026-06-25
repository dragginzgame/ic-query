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

pub(super) trait NnsNodeOperatorSource {
    fn fetch_node_operators(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeOperatorList, NnsNodeOperatorHostError>;
}

pub(super) struct LiveNnsNodeOperatorSource;

impl NnsNodeOperatorSource for LiveNnsNodeOperatorSource {
    fn fetch_node_operators(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeOperatorList, NnsNodeOperatorHostError> {
        Ok(fetch_mainnet_node_operator_list(request)?)
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
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_node_operators(&fetch_request)?;
    Ok(node_operator_report_from_list(list))
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
