use super::*;
use crate::nns::{LiveNnsSource, MAINNET_GOVERNANCE_CANISTER_ID, NnsInventoryCacheRequest};
use crate::report::CollectionSourceRequest;
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};

mod fixtures;
mod info;
mod list;
mod refresh;

#[tokio::test(flavor = "current_thread")]
async fn live_node_provider_source_rejects_non_mainnet_before_agent_construction() {
    let request = CollectionSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource::default()
        .fetch_node_provider_metadata(&request)
        .await
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsInventoryError::UnsupportedNetwork { collection: crate::nns::NnsInventoryCollection::NodeProvider, network } if network == "local"
    ));
}
