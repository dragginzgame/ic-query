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

pub(super) trait NnsNodeProviderSource {
    fn fetch_node_providers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError>;
}

pub(super) struct LiveNnsNodeProviderSource;

impl NnsNodeProviderSource for LiveNnsNodeProviderSource {
    fn fetch_node_providers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError> {
        Ok(fetch_mainnet_node_provider_list(request)?)
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
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_node_providers(&fetch_request)?;
    Ok(node_provider_report_from_list(list))
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
