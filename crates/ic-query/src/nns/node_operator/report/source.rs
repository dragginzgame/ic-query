use super::{
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorHostError,
    NnsNodeOperatorListReport, NnsNodeOperatorRow,
};
use crate::{
    ic_registry::{MainnetNodeOperatorList, fetch_mainnet_node_operator_list},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsNodeOperatorSource
///
/// Source contract for fetching complete NNS node-operator list reports.
///

pub trait NnsNodeOperatorSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError>;
}

impl NnsNodeOperatorSource for LiveNnsSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            NnsNodeOperatorHostError::UnsupportedNetwork { network }
        })?;
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
    let fetch_request = NnsSourceRequest::new(network, source_endpoint, fetched_at, "ic-query");
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
