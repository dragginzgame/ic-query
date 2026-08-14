//! Module: nns::governance_query
//!
//! Responsibility: perform typed query calls against mainnet NNS Governance.
//! Does not own: report-specific errors, wire projections, or cache policy.
//! Boundary: centralizes agent construction, Candid encoding, transport, decoding, and errors.

use crate::{
    agent::build_ic_agent_with_response_limit,
    nns::{
        MAINNET_GOVERNANCE_CANISTER_ID, NnsSourceRequest,
        governance::MAX_NNS_GOVERNANCE_RESPONSE_BYTES,
    },
};
use candid::{CandidType, Deserialize, Principal};
use thiserror::Error as ThisError;

///
/// NnsGovernanceQueryError
///
/// Transport failure from one typed NNS Governance query.
///

#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
pub enum NnsGovernanceQueryError {
    /// The IC agent could not be constructed for the requested endpoint.
    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild {
        /// Endpoint used to build the agent.
        endpoint: String,
        /// Agent construction failure.
        reason: String,
    },
    /// A query call to the NNS Governance canister failed.
    #[error("NNS Governance agent call {method} failed: {reason}")]
    AgentCall {
        /// Governance method being queried.
        method: &'static str,
        /// Agent call failure.
        reason: String,
    },
    /// A Governance query argument could not be Candid encoded.
    #[error("failed to encode Candid {message}: {reason}")]
    CandidEncode {
        /// Candid request type.
        message: &'static str,
        /// Encoding failure.
        reason: String,
    },
    /// A Governance query response could not be Candid decoded.
    #[error("failed to decode Candid {message}: {reason}")]
    CandidDecode {
        /// Candid response type.
        message: &'static str,
        /// Decoding failure.
        reason: String,
    },
}

pub async fn query_nns_governance<Arg, Response>(
    request: &NnsSourceRequest,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, NnsGovernanceQueryError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|error| NnsGovernanceQueryError::CandidEncode {
        message: request_message,
        reason: error.to_string(),
    })?;
    let bytes = query_nns_governance_bytes(request, method, arg).await?;
    decode_response(&bytes, response_message)
}

pub async fn query_nns_governance_no_args<Response>(
    request: &NnsSourceRequest,
    method: &'static str,
    response_message: &'static str,
) -> Result<Response, NnsGovernanceQueryError>
where
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_args(()).map_err(|error| NnsGovernanceQueryError::CandidEncode {
        message: "()",
        reason: error.to_string(),
    })?;
    let bytes = query_nns_governance_bytes(request, method, arg).await?;
    decode_response(&bytes, response_message)
}

async fn query_nns_governance_bytes(
    request: &NnsSourceRequest,
    method: &'static str,
    arg: Vec<u8>,
) -> Result<Vec<u8>, NnsGovernanceQueryError> {
    let agent = build_ic_agent_with_response_limit(
        &request.endpoint,
        |reason| NnsGovernanceQueryError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason,
        },
        MAX_NNS_GOVERNANCE_RESPONSE_BYTES,
    )?;
    let canister = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|error| {
        NnsGovernanceQueryError::CandidDecode {
            message: "governance_canister_id",
            reason: error.to_string(),
        }
    })?;
    agent
        .query(&canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|error| NnsGovernanceQueryError::AgentCall {
            method,
            reason: error.to_string(),
        })
}

fn decode_response<Response>(
    bytes: &[u8],
    response_message: &'static str,
) -> Result<Response, NnsGovernanceQueryError>
where
    Response: for<'de> Deserialize<'de> + CandidType,
{
    candid::decode_one(bytes).map_err(|error| NnsGovernanceQueryError::CandidDecode {
        message: response_message,
        reason: error.to_string(),
    })
}
