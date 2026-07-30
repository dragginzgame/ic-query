//! Module: nns::governance_query
//!
//! Responsibility: perform typed query calls against mainnet NNS Governance.
//! Does not own: report-specific errors, wire projections, or cache policy.
//! Boundary: centralizes agent construction, Candid encoding, transport, and decoding.

use crate::{ic_registry::MAINNET_GOVERNANCE_CANISTER_ID, nns::NnsSourceRequest};
use candid::{CandidType, Deserialize, Principal};
use ic_agent::Agent;

///
/// NnsGovernanceQueryError
///
/// Internal transport failure from one typed NNS Governance query.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NnsGovernanceQueryError {
    AgentBuild {
        endpoint: String,
        reason: String,
    },
    AgentCall {
        method: &'static str,
        reason: String,
    },
    CandidEncode {
        message: &'static str,
        reason: String,
    },
    CandidDecode {
        message: &'static str,
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
    let agent = Agent::builder()
        .with_url(&request.endpoint)
        .build()
        .map_err(|error| NnsGovernanceQueryError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason: error.to_string(),
        })?;
    let canister = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|error| {
        NnsGovernanceQueryError::CandidDecode {
            message: "governance_canister_id",
            reason: error.to_string(),
        }
    })?;
    let arg = candid::encode_one(arg).map_err(|error| NnsGovernanceQueryError::CandidEncode {
        message: request_message,
        reason: error.to_string(),
    })?;
    let bytes = agent
        .query(&canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|error| NnsGovernanceQueryError::AgentCall {
            method,
            reason: error.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|error| NnsGovernanceQueryError::CandidDecode {
        message: response_message,
        reason: error.to_string(),
    })
}
