use super::*;
use crate::ic_registry::{MAINNET_GOVERNANCE_CANISTER_ID, MainnetNodeProvider};
use crate::nns::{LiveNnsSource, NnsSourceRequest};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};

mod fixtures;
mod info;
mod list;
mod refresh;
mod text;

#[test]
fn live_node_provider_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource
        .fetch_node_provider_list_report(&request)
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsNodeProviderHostError::UnsupportedNetwork { network } if network == "local"
    ));
}
