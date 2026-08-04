//! Module: sns::report::live::fetch::upgrade
//!
//! Responsibility: fetch one SNS running version and its next blessed version.
//! Does not own: discovery, source validation, report assembly, cache IO, or rendering.
//! Boundary: performs exactly two bounded native queries and retains only next-version failure.

use super::block_on_sns;
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsUpgrade, SnsCanisterMethod, SnsHostError,
    SnsRunningVersionResponse, SnsSourceRequest, SnsUpgradeQueryGap,
    live::{
        convert::{sns_pending_upgrade, sns_running_version_response, sns_version},
        query::{principal_from_text, query_canister, sns_agent},
        types::{
            GetNextSnsVersionRequest, GetNextSnsVersionResponse, GetRunningSnsVersionRequest,
            GetRunningSnsVersionResponse,
        },
    },
};
use candid::Principal;
use ic_agent::Agent;

/// Fetch the complete native Governance running-version response.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_running_version(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsRunningVersionResponse, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_running_version_async(request, sns))
}

/// Fetch bounded native upgrade-version state for one resolved SNS.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_upgrade(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsUpgrade, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_upgrade_async(request, sns))
}

async fn fetch_mainnet_sns_upgrade_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsUpgrade, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;

    let running = query_running_sns_version(&agent, &governance_canister).await?;
    let deployed_wire =
        running
            .deployed_version
            .ok_or_else(|| SnsHostError::MissingRunningSnsVersion {
                method: SnsCanisterMethod::GetRunningSnsVersion.as_str(),
                governance_canister_id: sns.governance_canister_id.clone(),
            })?;
    let deployed_version = sns_version(deployed_wire.clone());
    let pending_upgrade = running.pending_version.map(sns_pending_upgrade);

    let next_request = GetNextSnsVersionRequest {
        governance_canister_id: Some(governance_canister),
        current_version: Some(deployed_wire),
    };
    let (next_version, next_version_gap) = match query_canister::<_, GetNextSnsVersionResponse>(
        &agent,
        &sns_wasm_canister,
        SnsCanisterMethod::GetNextSnsVersion.as_str(),
        "GetNextSnsVersionRequest",
        "GetNextSnsVersionResponse",
        &next_request,
    )
    .await
    {
        Ok(response) => (response.next_version.map(sns_version), None),
        Err(error) => {
            let reason = error.to_string();
            (
                None,
                Some(SnsUpgradeQueryGap {
                    method: SnsCanisterMethod::GetNextSnsVersion,
                    reason: reason.trim().to_string(),
                }),
            )
        }
    };

    Ok(MainnetSnsUpgrade {
        governance_canister_id: sns.governance_canister_id.clone(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        running_version_method: SnsCanisterMethod::GetRunningSnsVersion,
        next_version_method: SnsCanisterMethod::GetNextSnsVersion,
        point_in_time_guaranteed: false,
        deployed_version,
        pending_upgrade,
        next_version,
        next_version_gap,
    })
}

async fn fetch_mainnet_sns_running_version_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsRunningVersionResponse, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let response = query_running_sns_version(&agent, &governance_canister).await?;
    Ok(sns_running_version_response(response))
}

async fn query_running_sns_version(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetRunningSnsVersionResponse, SnsHostError> {
    query_canister(
        agent,
        governance_canister,
        SnsCanisterMethod::GetRunningSnsVersion.as_str(),
        "GetRunningSnsVersionRequest",
        "GetRunningSnsVersionResponse",
        &GetRunningSnsVersionRequest {},
    )
    .await
}
