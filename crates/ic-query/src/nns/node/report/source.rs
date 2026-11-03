use super::{NNS_NODE_LIST_REPORT_SCHEMA_VERSION, NnsInventoryError, NnsNodeListReport};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_node_list,
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeSource
///
/// Source contract for fetching complete NNS node list reports.
///

pub trait NnsNodeSource: Sync {
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsInventoryError>>;
}

impl NnsNodeSource for LiveNnsSource {
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeListReport, NnsInventoryError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: super::COLLECTION,
                    network,
                }
            })?;
            let (provenance, nodes) =
                fetch_mainnet_node_list(&fetch_request)
                    .await
                    .map_err(|source| NnsInventoryError::Registry {
                        collection: super::COLLECTION,
                        source,
                    })?;
            Ok(NnsNodeListReport {
                schema_version: NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
                provenance,
                nodes,
            })
        })
    }
}
