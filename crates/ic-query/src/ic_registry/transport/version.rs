use super::{RegistryQueryCounter, decode_message};
use crate::ic_registry::{RegistryFetchError, proto::RegistryGetLatestVersionResponse};
use candid::Principal;
use ic_agent::Agent;

#[cfg(feature = "nns-topology-host")]
pub(in crate::ic_registry) async fn get_latest_version(
    agent: &Agent,
    registry_canister: &Principal,
) -> Result<u64, RegistryFetchError> {
    get_latest_version_inner(agent, registry_canister, None).await
}

pub(in crate::ic_registry) async fn get_latest_version_counted(
    agent: &Agent,
    registry_canister: &Principal,
    counter: &RegistryQueryCounter,
) -> Result<u64, RegistryFetchError> {
    get_latest_version_inner(agent, registry_canister, Some(counter)).await
}

async fn get_latest_version_inner(
    agent: &Agent,
    registry_canister: &Principal,
    counter: Option<&RegistryQueryCounter>,
) -> Result<u64, RegistryFetchError> {
    if let Some(counter) = counter {
        counter.record_call();
    }
    let bytes = agent
        .query(registry_canister, "get_latest_version")
        .with_arg(Vec::<u8>::new())
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "get_latest_version",
            reason: err.to_string(),
        })?;
    let response = decode_message::<RegistryGetLatestVersionResponse>(
        "RegistryGetLatestVersionResponse",
        &bytes,
    )?;
    Ok(response.version)
}
