//! Module: sns::report::live::query
//!
//! Responsibility: low-level live SNS query helpers.
//! Does not own: report assembly, Candid wire type definitions, or rendering.
//! Boundary: builds agents, parses principals, and maps query failures to typed errors.

use super::SnsHostError;
use crate::report::CollectionSourceRequest;
use crate::{agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES, sns::report::enforce_mainnet_network};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;

crate::candid_call::impl_candid_call_error!(SnsHostError);

/// Build an IC agent after enforcing one explicit SNS source request.
pub(super) fn sns_agent<'a>(
    source: &'a crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
) -> Result<&'a Agent, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    crate::agent::endpoint_agent(
        &source.agent,
        &request.endpoint,
        MAX_IC_AGENT_RESPONSE_BODY_BYTES,
        |reason| SnsHostError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason,
        },
    )
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
