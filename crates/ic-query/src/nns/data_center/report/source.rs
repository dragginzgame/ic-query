use super::{
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::{MainnetDataCenterList, fetch_mainnet_data_center_list},
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
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsDataCenterHostError>>;
}

impl NnsDataCenterSource for LiveNnsSource {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsDataCenterHostError>>
    {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsDataCenterHostError::UnsupportedNetwork { network }
            })?;
            Ok(data_center_report_from_list(
                fetch_mainnet_data_center_list(&fetch_request).await?,
            ))
        })
    }
}

fn data_center_report_from_list(list: MainnetDataCenterList) -> NnsDataCenterListReport {
    let data_centers = list.data_centers;
    NnsDataCenterListReport {
        schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        provenance: list.provenance,
        data_center_count: data_centers.len(),
        data_centers,
    }
}
