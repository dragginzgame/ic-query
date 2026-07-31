//! Module: agent
//!
//! Responsibility: validate live source endpoints before constructing IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use crate::http_endpoint::parse_http_endpoint;
use ic_agent::Agent;

/// Validate an HTTP(S) endpoint and construct an IC agent without parser panics.
pub fn build_ic_agent<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
) -> Result<Agent, Error> {
    parse_http_endpoint(endpoint).map_err(&map_error)?;

    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|error| map_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_agent_for_valid_endpoint_without_network_io() {
        build_ic_agent("https://icp-api.io", |reason| reason)
            .expect("valid HTTPS endpoint must construct an agent");
    }
}
