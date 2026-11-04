//! Module: agent
//!
//! Responsibility: validate endpoints before constructing response-bounded IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use crate::http_endpoint::parse_http_endpoint;
#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
use ic_host_tools::agent::{EndpointAgent, EndpointAgentError};
use ic_platform::agent::Agent;

/// Maximum response-body size configured for every native IC agent call.
pub const MAX_IC_AGENT_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Admit Query's endpoint policy before selecting a Host-owned transport.
#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
pub fn endpoint_agent<'a, Error>(
    binding: &'a EndpointAgent,
    endpoint: &str,
    maximum_response_bytes: usize,
    map_error: impl Fn(String) -> Error,
) -> Result<&'a Agent, Error> {
    parse_http_endpoint(endpoint).map_err(&map_error)?;
    binding
        .get(endpoint, maximum_response_bytes)
        .map_err(|error| {
            map_error(match error {
                EndpointAgentError::Endpoint(source) => source.to_string(),
                EndpointAgentError::Build(source) => source.to_string(),
                EndpointAgentError::BindingMismatch => error.to_string(),
            })
        })
}

/// Validate a clean HTTP(S) endpoint and construct one response-bounded IC agent.
#[cfg(any(
    feature = "ic-state-host",
    feature = "cmc-host",
    feature = "subnet-catalog-host",
    test
))]
pub fn build_ic_agent<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
) -> Result<Agent, Error> {
    let endpoint = parse_http_endpoint(endpoint).map_err(&map_error)?;
    ic_host_tools::agent::agent_builder(&endpoint, MAX_IC_AGENT_RESPONSE_BODY_BYTES)
        .build()
        .map_err(|error| map_error(error.to_string()))
}

#[cfg(test)]
mod tests;
