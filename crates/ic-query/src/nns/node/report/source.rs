use super::{NNS_NODE_LIST_REPORT_SCHEMA_VERSION, NnsNodeHostError, NnsNodeListReport, NnsNodeRow};
use crate::{
    ic_registry::{MainnetNodeList, fetch_mainnet_node_list},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeSource
///
/// Source contract for fetching complete NNS node list reports.
///

pub trait NnsNodeSource: Sync {
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsNodeHostError>>;
}

impl NnsNodeSource for LiveNnsSource {
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsNodeHostError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsNodeHostError::UnsupportedNetwork { network }
            })?;
            Ok(node_report_from_list(
                fetch_mainnet_node_list(&fetch_request).await?,
            ))
        })
    }
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
