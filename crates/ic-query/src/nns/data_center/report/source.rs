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

///
/// NnsDataCenterSourceRequest
///
/// Source request settings for fetching one complete NNS data-center snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterSourceRequest {
    pub network: String,
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsDataCenterSourceRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// NnsDataCenterSource
///
/// Source contract for fetching complete NNS data-center list reports.
///

pub trait NnsDataCenterSource {
    fn fetch_data_center_list_report(
        &self,
        request: &NnsDataCenterSourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsDataCenterHostError>;
}

///
/// LiveNnsDataCenterSource
///
/// Source implementation backed by live NNS registry data-center calls.
///

pub struct LiveNnsDataCenterSource;

impl NnsDataCenterSource for LiveNnsDataCenterSource {
    fn fetch_data_center_list_report(
        &self,
        request: &NnsDataCenterSourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        Ok(data_center_report_from_list(
            fetch_mainnet_data_center_list(&fetch_request)?,
        ))
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
    let fetch_request =
        NnsDataCenterSourceRequest::new(network, source_endpoint, fetched_at, "ic-query");
    source.fetch_data_center_list_report(&fetch_request)
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
