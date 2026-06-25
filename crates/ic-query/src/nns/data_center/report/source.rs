use super::{
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
    NnsDataCenterRow,
};
use crate::{
    ic_registry::{
        MainnetDataCenterList, MainnetRegistryFetchRequest, fetch_mainnet_data_center_list,
    },
    subnet_catalog::format_utc_timestamp_secs,
};

pub(super) trait NnsDataCenterSource {
    fn fetch_data_centers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetDataCenterList, NnsDataCenterHostError>;
}

pub(super) struct LiveNnsDataCenterSource;

impl NnsDataCenterSource for LiveNnsDataCenterSource {
    fn fetch_data_centers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetDataCenterList, NnsDataCenterHostError> {
        Ok(fetch_mainnet_data_center_list(request)?)
    }
}

pub(super) fn fetch_nns_data_center_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    super::enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_data_centers(&fetch_request)?;
    Ok(data_center_report_from_list(list))
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
