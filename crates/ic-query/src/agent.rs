//! Module: agent
//!
//! Responsibility: validate live source endpoints before constructing IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use ic_agent::Agent;
use url::Url;

/// Validate an HTTP(S) endpoint and construct an IC agent without parser panics.
pub fn build_ic_agent<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
) -> Result<Agent, Error> {
    let parsed = Url::parse(endpoint).map_err(|error| map_error(error.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(map_error(format!(
            "unsupported URL scheme {:?}; expected http or https",
            parsed.scheme()
        )));
    }
    if parsed.host_str().is_none() {
        return Err(map_error("endpoint URL must include a host".to_string()));
    }

    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|error| map_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_malformed_endpoint_without_panicking() {
        let error = build_ic_agent(":::", |reason| reason)
            .expect_err("malformed endpoint must be rejected");

        assert!(!error.is_empty());
    }

    #[test]
    fn rejects_non_http_endpoint() {
        let error = build_ic_agent("ftp://example.com", |reason| reason)
            .expect_err("unsupported URL scheme must be rejected");

        assert!(error.contains("expected http or https"));
    }

    #[test]
    fn constructs_agent_for_valid_endpoint_without_network_io() {
        build_ic_agent("https://icp-api.io", |reason| reason)
            .expect("valid HTTPS endpoint must construct an agent");
    }
}
