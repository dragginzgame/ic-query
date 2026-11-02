use super::{
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterListReport, NnsInventoryError,
};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_data_center_list,
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};

///
/// NnsDataCenterSource
///
/// Source contract for fetching complete NNS data-center list reports.
///

pub trait NnsDataCenterSource: Sync {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsInventoryError>>;
}

impl NnsDataCenterSource for LiveNnsSource {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsInventoryError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: super::COLLECTION,
                    network,
                }
            })?;
            let (provenance, data_centers) = fetch_mainnet_data_center_list(&fetch_request)
                .await
                .map_err(|source| NnsInventoryError::Registry {
                collection: super::COLLECTION,
                source,
            })?;
            Ok(NnsDataCenterListReport {
                schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
                provenance,
                data_center_count: data_centers.len(),
                data_centers,
            })
        })
    }
}
