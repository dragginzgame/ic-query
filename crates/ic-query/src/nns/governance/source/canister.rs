//! Module: nns::governance::source::canister
//!
//! Responsibility: call NNS Governance from replicated canister execution.
//! Does not own: scheduling, retries, persistence, or report assembly.
//! Boundary: this is the only module coupled to the IC canister runtime.

use super::GovernanceQuery;
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::{
        NnsGovernanceError, NnsGovernanceRequest, NnsGovernanceSourceProvenance,
        NnsGovernanceSourceSelection,
        validation::{validate_governance_request, validate_governance_response_size},
    },
};
use ic_platform::Principal;
use ic_platform::candid::contract::{ArgumentEncoder, CandidType, serde::Deserialize};
use ic_platform::cdk::{
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

impl GovernanceQuery for CanisterNnsSource {
    type Request = ();

    fn prepare(
        &self,
        request: &NnsGovernanceRequest,
    ) -> Result<(Self::Request, NnsGovernanceSourceProvenance), NnsGovernanceError> {
        Ok(((), canister_provenance(request)?))
    }

    async fn query<Args, Response>(
        &self,
        _request: &(),
        method: &'static str,
        request_message: &'static str,
        response_message: &'static str,
        args: Args,
    ) -> Result<Response, NnsGovernanceError>
    where
        Args: ArgumentEncoder + Send,
        Response: for<'de> Deserialize<'de> + CandidType,
    {
        let arg = ic_platform::candid::contract::encode_args(args).map_err(|error| {
            NnsGovernanceError::CandidEncode {
                message: request_message,
                reason: error.to_string(),
            }
        })?;
        let bytes = call_bytes(method, arg).await?;
        decode_response(&bytes, response_message)
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
    crate::candid_decode::decode_reply(bytes).map_err(|error| NnsGovernanceError::CandidDecode {
        message: response_message,
        reason: error.to_string(),
    })
}
