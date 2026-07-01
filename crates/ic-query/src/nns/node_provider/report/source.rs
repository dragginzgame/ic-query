use super::{
    NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderHostError,
    NnsNodeProviderListReport, NnsNodeProviderRow,
};
use crate::{
    ic_registry::{
        MainnetNodeProviderList, MainnetRegistryFetchRequest, fetch_mainnet_node_provider_list,
    },
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// NnsNodeProviderSourceRequest
///
/// Source request settings for fetching one complete NNS node-provider snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderSourceRequest {
    pub network: String,
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsNodeProviderSourceRequest {
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
/// NnsNodeProviderSource
///
/// Source contract for fetching complete NNS node-provider list reports.
///

pub trait NnsNodeProviderSource {
    fn fetch_node_provider_list_report(
        &self,
        request: &NnsNodeProviderSourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError>;
}

///
/// LiveNnsNodeProviderSource
///
/// Source implementation backed by live NNS governance and registry calls.
///

pub struct LiveNnsNodeProviderSource;

impl NnsNodeProviderSource for LiveNnsNodeProviderSource {
    fn fetch_node_provider_list_report(
        &self,
        request: &NnsNodeProviderSourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        Ok(node_provider_report_from_list(
            fetch_mainnet_node_provider_list(&fetch_request)?,
        ))
    }
}

pub(super) fn fetch_nns_node_provider_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    super::enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let fetch_request =
        NnsNodeProviderSourceRequest::new(network, source_endpoint, fetched_at, "ic-query");
    source.fetch_node_provider_list_report(&fetch_request)
}

fn node_provider_report_from_list(list: MainnetNodeProviderList) -> NnsNodeProviderListReport {
    let node_providers = list
        .node_providers
        .into_iter()
        .map(|provider| NnsNodeProviderRow {
            node_provider_principal: provider.principal,
            name: None,
            node_count: provider.node_count,
            reward_account_hex: provider.reward_account_hex,
        })
        .collect::<Vec<_>>();
    NnsNodeProviderListReport {
        schema_version: NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        governance_canister_id: list.governance_canister_id,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_provider_count: node_providers.len(),
        node_providers,
    }
}
