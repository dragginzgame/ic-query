use super::{
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorHostError,
    NnsNodeOperatorListReport, NnsNodeOperatorRow,
};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::{MainnetNodeOperatorList, fetch_mainnet_node_operator_list},
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeOperatorSource
///
/// Source contract for fetching complete NNS node-operator list reports.
///

pub trait NnsNodeOperatorSource: Sync {
    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError>>;
}

impl NnsNodeOperatorSource for LiveNnsSource {
    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError>>
    {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsNodeOperatorHostError::UnsupportedNetwork { network }
            })?;
            Ok(node_operator_report_from_list(
                fetch_mainnet_node_operator_list(&fetch_request).await?,
            ))
        })
    }
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
