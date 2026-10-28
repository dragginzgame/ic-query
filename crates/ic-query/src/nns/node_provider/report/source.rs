use super::{
    NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderHostError,
    NnsNodeProviderListReport, NnsNodeProviderRow,
};
use crate::{
    ic_registry::{MainnetNodeProviderList, fetch_mainnet_node_provider_list},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeProviderSource
///
/// Source contract for fetching complete NNS node-provider list reports.
///

pub trait NnsNodeProviderSource: Sync {
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsNodeProviderHostError>>;
}

impl NnsNodeProviderSource for LiveNnsSource {
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a NnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsNodeProviderHostError>>
    {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsNodeProviderHostError::UnsupportedNetwork { network }
            })?;
            Ok(node_provider_report_from_list(
                fetch_mainnet_node_provider_list(&fetch_request).await?,
            ))
        })
    }
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
