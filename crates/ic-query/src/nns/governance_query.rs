//! Module: nns::governance_query
//!
//! Responsibility: perform typed query calls against mainnet NNS Governance.
//! Does not own: report-specific errors, wire projections, or cache policy.
//! Boundary: retains NNS agent policy while sharing Candid query mechanics.

use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID, NnsSourceRequest,
    governance::{MAX_NNS_GOVERNANCE_RESPONSE_BYTES, NnsGovernanceError},
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::{CandidType, Principal, serde::Deserialize};

pub async fn query_nns_governance<Arg, Response>(
    source: &crate::nns::LiveNnsSource,
    request: &NnsSourceRequest,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, NnsGovernanceError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = crate::candid_call::encode_request((arg,), request_message)?;
    let (agent, canister) = governance_target(source, request)?;
    crate::candid_call::query_encoded(agent, &canister, method, response_message, arg).await
}

pub async fn query_nns_governance_no_args<Response>(
    source: &crate::nns::LiveNnsSource,
    request: &NnsSourceRequest,
    method: &'static str,
    response_message: &'static str,
) -> Result<Response, NnsGovernanceError>
where
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = crate::candid_call::encode_request((), "()")?;
    let (agent, canister) = governance_target(source, request)?;
    crate::candid_call::query_encoded(agent, &canister, method, response_message, arg).await
}

fn governance_target<'a>(
    source: &'a crate::nns::LiveNnsSource,
    request: &NnsSourceRequest,
) -> Result<(&'a Agent, Principal), NnsGovernanceError> {
    let agent = crate::agent::endpoint_agent(
        &source.agent,
        &request.endpoint,
        MAX_NNS_GOVERNANCE_RESPONSE_BYTES,
        |reason| NnsGovernanceError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason,
        },
    )?;
    let canister = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|error| {
        NnsGovernanceError::CandidDecode {
            message: "governance_canister_id",
            reason: error.to_string(),
        }
    })?;
    Ok((agent, canister))
}
