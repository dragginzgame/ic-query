use super::{
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
    NnsDataCenterRow,
};
use crate::{
    ic_registry::{MainnetDataCenterList, fetch_mainnet_data_center_list},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
};

///
/// NnsDataCenterSource
///
/// Source contract for fetching complete NNS data-center list reports.
///

pub trait NnsDataCenterSource: Sync {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsDataCenterHostError>>;
}

impl NnsDataCenterSource for LiveNnsSource {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
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
    let data_centers = list
        .data_centers
        .into_iter()
        .map(|data_center| NnsDataCenterRow {
            data_center_id: data_center.id,
            region: data_center.region,
            owner: data_center.owner,
            latitude: data_center.latitude,
            longitude: data_center.longitude,
            node_operator_count: data_center.node_operator_count,
            node_provider_count: data_center.node_provider_count,
            node_count: data_center.node_count,
        })
        .collect::<Vec<_>>();
    NnsDataCenterListReport {
        schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        data_center_count: data_centers.len(),
        data_centers,
    }
}
