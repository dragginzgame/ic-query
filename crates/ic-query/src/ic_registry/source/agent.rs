#[cfg(feature = "nns-host")]
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
use crate::{
    agent::build_ic_agent,
    ic_registry::{MainnetRegistryFetchRequest, RegistryFetchError},
    subnet_catalog::MAINNET_REGISTRY_CANISTER_ID,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;

pub(in crate::ic_registry::source) fn mainnet_agent(
    request: &MainnetRegistryFetchRequest,
) -> Result<Agent, RegistryFetchError> {
    build_ic_agent(&request.endpoint, |reason| RegistryFetchError::AgentBuild {
        endpoint: request.endpoint.clone(),
        reason,
    })
}

pub(in crate::ic_registry::source) fn mainnet_registry_canister()
-> Result<Principal, RegistryFetchError> {
    principal_from_text(MAINNET_REGISTRY_CANISTER_ID, "registry_canister_id")
}

#[cfg(feature = "nns-host")]
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
