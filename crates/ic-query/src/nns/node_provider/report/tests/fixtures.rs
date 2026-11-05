use super::*;
use crate::report::CollectionSourceRequest;
use crate::tests::temp_dir;

pub(super) fn node_provider_report_fixture() -> NnsNodeProviderListReport {
    NnsNodeProviderListReport {
        governance_source: crate::report::CollectionSourceRequest::from_unix_secs(
            "ic",
            "https://icp-api.io",
            1_780_531_200,
            "fixture",
        ),
        schema_version: 1,
        provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: "2026-06-04T00:00:00Z".to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            fetched_by: "test".to_string(),
        },
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        node_providers: vec![
            NnsNodeProviderRow {
                node_provider_principal: "aaaaa-aa".to_string(),
                name: None,
                node_count: Some(3),
                reward_account_hex: None,
            },
            NnsNodeProviderRow {
                node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                name: Some("DFINITY".to_string()),
                node_count: Some(13),
                reward_account_hex: Some("abcd".to_string()),
            },
        ],
    }
}

///
/// FixtureNodeProviderSource
///
/// Fixture Registry counts and independent Governance metadata.
///

pub(super) struct FixtureNodeProviderSource {
    pub(super) node_providers: Vec<NnsNodeProviderRow>,
}
impl crate::nns::NnsRegistryInventorySource for FixtureNodeProviderSource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::NnsRegistryInventorySnapshot, NnsInventoryError>,
    > {
        Box::pin(async move {
            Ok(crate::tests::inventory_for_provider_rows(
                request,
                &self.node_providers,
            ))
        })
    }
}
impl NnsNodeProviderSource for FixtureNodeProviderSource {
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderMetadata, NnsInventoryError>> {
        Box::pin(async move {
            let mut node_providers = self.node_providers.clone();
            for row in &mut node_providers {
                row.node_count = None;
            }
            Ok(NnsNodeProviderMetadata {
                schema_version: 1,
                source: request.clone(),
                governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
                node_providers,
            })
        })
    }
}
///
/// FailingNodeProviderSource
///
/// Reject live calls to prove complete-cache reuse.
///

pub(super) struct FailingNodeProviderSource;
impl crate::nns::NnsRegistryInventorySource for FailingNodeProviderSource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::NnsRegistryInventorySnapshot, NnsInventoryError>,
    > {
        Box::pin(async move {
            Err(NnsInventoryError::NotFound {
                collection: crate::nns::NnsInventoryCollection::NodeProvider,
                input: "unexpected-live-fetch".to_string(),
            })
        })
    }
}
impl NnsNodeProviderSource for FailingNodeProviderSource {
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderMetadata, NnsInventoryError>> {
        Box::pin(async move {
            Err(NnsInventoryError::NotFound {
                collection: crate::nns::NnsInventoryCollection::NodeProvider,
                input: "unexpected-live-fetch".to_string(),
            })
        })
    }
}
pub(super) fn test_cache_request(network: &str, name: &str) -> NnsInventoryCacheRequest {
    NnsInventoryCacheRequest {
        cache_root: temp_dir(&format!("ic-query-nns-node-provider-{name}")),
        network: network.to_string(),
    }
}
