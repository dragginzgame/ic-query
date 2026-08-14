//! Module: nns::governance::source::canister
//!
//! Responsibility: call NNS Governance from replicated canister execution.
//! Does not own: scheduling, retries, persistence, or report assembly.
//! Boundary: this is the only module coupled to the IC canister runtime.

use super::{
    NnsGovernanceSource, NnsGovernanceSourceData, NnsGovernanceSourceFuture, metrics_result,
};
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::{
        NnsGovernanceEconomics, NnsGovernanceError, NnsGovernanceMaturityModulation,
        NnsGovernanceMetrics, NnsGovernanceRequest, NnsGovernanceRewardEvent,
        NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
        validation::{validate_governance_request, validate_governance_response_size},
        wire::{GetMaturityModulationRequest, GetMaturityModulationResponse, GetMetricsResult},
    },
};
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{
    api::canister_self,
    call::{Call, CallFailed},
};

///
/// CanisterNnsSource
///
/// Built-in replicated inter-canister source for direct NNS Governance reports.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct CanisterNnsSource;

impl NnsGovernanceSource for CanisterNnsSource {
    fn fetch_economics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceEconomics> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let economics =
                call_no_args("get_network_economics_parameters", "NetworkEconomics").await?;
            Ok(NnsGovernanceSourceData::new(economics, provenance))
        })
    }

    fn fetch_metrics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceMetrics> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let result: GetMetricsResult = call_no_args("get_metrics", "GetMetricsResult").await?;
            Ok(NnsGovernanceSourceData::new(
                NnsGovernanceMetrics::from(metrics_result(result)?),
                provenance,
            ))
        })
    }

    fn fetch_reward_event<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, NnsGovernanceRewardEvent> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let reward_event = call_no_args("get_latest_reward_event", "RewardEvent").await?;
            Ok(NnsGovernanceSourceData::new(reward_event, provenance))
        })
    }

    fn fetch_maturity_modulation<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let response: GetMaturityModulationResponse = call_with_arg(
                "get_maturity_modulation",
                "GetMaturityModulationRequest",
                "GetMaturityModulationResponse",
                &GetMaturityModulationRequest {},
            )
            .await?;
            Ok(NnsGovernanceSourceData::new(
                response.maturity_modulation,
                provenance,
            ))
        })
    }
}

fn canister_provenance(
    request: &NnsGovernanceRequest,
) -> Result<NnsGovernanceSourceProvenance, NnsGovernanceError> {
    validate_governance_request(request)?;
    if request.source != NnsGovernanceSourceSelection::ReplicatedInterCanisterCall {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: "the canister adapter requires a replicated_inter_canister_call source"
                .to_string(),
        });
    }
    Ok(NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
        collector_canister_id: canister_self().to_text(),
    })
}

async fn call_no_args<Response>(
    method: &'static str,
    response_message: &'static str,
) -> Result<Response, NnsGovernanceError>
where
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_args(()).map_err(|error| NnsGovernanceError::CandidEncode {
        message: "()",
        reason: error.to_string(),
    })?;
    let bytes = call_bytes(method, arg).await?;
    decode_response(&bytes, response_message)
}

async fn call_with_arg<Arg, Response>(
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, NnsGovernanceError>
where
    Arg: CandidType,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|error| NnsGovernanceError::CandidEncode {
        message: request_message,
        reason: error.to_string(),
    })?;
    let bytes = call_bytes(method, arg).await?;
    decode_response(&bytes, response_message)
}

async fn call_bytes(method: &'static str, arg: Vec<u8>) -> Result<Vec<u8>, NnsGovernanceError> {
    let canister_id = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|error| {
        NnsGovernanceError::CandidDecode {
            message: "governance_canister_id",
            reason: error.to_string(),
        }
    })?;
    let response = Call::bounded_wait(canister_id, method)
        .take_raw_args(arg)
        .await
        .map_err(|error| map_call_error(method, error))?;
    let bytes = response.into_bytes();
    validate_governance_response_size(method, bytes.len())?;
    Ok(bytes)
}

fn map_call_error(method: &'static str, error: CallFailed) -> NnsGovernanceError {
    match error {
        CallFailed::CallRejected(error) => NnsGovernanceError::InterCanisterCallRejected {
            method,
            reject_code: error.raw_reject_code(),
            message: error.reject_message().to_string(),
        },
        error => NnsGovernanceError::InterCanisterCall {
            method,
            reason: error.to_string(),
        },
    }
}

fn decode_response<Response>(
    bytes: &[u8],
    response_message: &'static str,
) -> Result<Response, NnsGovernanceError>
where
    Response: for<'de> Deserialize<'de> + CandidType,
{
    candid::decode_one(bytes).map_err(|error| NnsGovernanceError::CandidDecode {
        message: response_message,
        reason: error.to_string(),
    })
}
