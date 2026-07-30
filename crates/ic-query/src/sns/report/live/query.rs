//! Module: sns::report::live::query
//!
//! Responsibility: low-level live SNS query helpers.
//! Does not own: report assembly, Candid wire type definitions, or rendering.
//! Boundary: builds agents, parses principals, and maps query failures to typed errors.

use super::SnsHostError;
use crate::{
    icrc::ledger::IcrcLedgerError,
    sns::report::{SnsSourceRequest, enforce_mainnet_network},
};
use candid::{CandidType, Deserialize, Principal};
use ic_agent::Agent;

impl IcrcLedgerError for SnsHostError {
    fn agent_build(endpoint: &str, reason: String) -> Self {
        Self::AgentBuild {
            endpoint: endpoint.to_string(),
            reason,
        }
    }

    fn invalid_principal(field: &'static str, reason: String) -> Self {
        Self::InvalidPrincipal { field, reason }
    }

    fn candid_encode(message: &'static str, reason: String) -> Self {
        Self::CandidEncode { message, reason }
    }

    fn agent_call(method: &'static str, reason: String) -> Self {
        Self::AgentCall { method, reason }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        Self::CandidDecode { message, reason }
    }
}

/// Query one Candid canister method with an explicit request payload.
pub(super) async fn query_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, SnsHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| SnsHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
    })
}

/// Call one Candid canister ingress method and wait for its response.
pub(super) async fn update_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, SnsHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| SnsHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .update(canister, method)
        .with_arg(arg)
        .call_and_wait()
        .await
        .map_err(|err| SnsHostError::AgentUpdateCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
    })
}

/// Build an IC agent after enforcing one explicit SNS source request.
pub(super) fn sns_agent(request: &SnsSourceRequest) -> Result<Agent, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    Agent::builder()
        .with_url(&request.endpoint)
        .build()
        .map_err(|err| SnsHostError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason: err.to_string(),
        })
}

/// Parse a principal text field into a typed principal or host error.
pub(super) fn principal_from_text(
    value: &str,
    field: &'static str,
) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}
