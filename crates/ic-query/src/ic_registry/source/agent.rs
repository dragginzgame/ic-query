use crate::{
    ic_registry::{
        MAINNET_GOVERNANCE_CANISTER_ID, MainnetRegistryFetchRequest, RegistryFetchError,
    },
    subnet_catalog::MAINNET_REGISTRY_CANISTER_ID,
};
use candid::Principal;
use ic_agent::Agent;

pub(in crate::ic_registry::source) fn mainnet_agent(
    request: &MainnetRegistryFetchRequest,
) -> Result<Agent, RegistryFetchError> {
    Agent::builder()
        .with_url(&request.endpoint)
        .build()
        .map_err(|err| RegistryFetchError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason: err.to_string(),
        })
}

pub(in crate::ic_registry::source) fn mainnet_registry_canister()
-> Result<Principal, RegistryFetchError> {
    principal_from_text(MAINNET_REGISTRY_CANISTER_ID, "registry_canister_id")
}

pub(in crate::ic_registry::source) fn mainnet_governance_canister()
-> Result<Principal, RegistryFetchError> {
    principal_from_text(MAINNET_GOVERNANCE_CANISTER_ID, "governance_canister_id")
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, RegistryFetchError> {
    Principal::from_text(value).map_err(|err| RegistryFetchError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}
