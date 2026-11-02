use super::{
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsInventoryError, NnsNodeOperatorListReport,
};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_node_operator_list,
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
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsInventoryError>>;
}

impl NnsNodeOperatorSource for LiveNnsSource {
    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsInventoryError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: super::COLLECTION,
                    network,
                }
            })?;
            let (provenance, node_operators) = fetch_mainnet_node_operator_list(&fetch_request)
                .await
                .map_err(|source| NnsInventoryError::Registry {
                    collection: super::COLLECTION,
                    source,
                })?;
            Ok(NnsNodeOperatorListReport {
                schema_version: NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
                provenance,
                node_operator_count: node_operators.len(),
                node_operators,
            })
        })
    }
}
