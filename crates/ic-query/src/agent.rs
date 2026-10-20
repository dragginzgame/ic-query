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
#[cfg(feature = "certified-subnet-catalog-host")]
use std::time::Duration;

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
    build_ic_agent_with_options(endpoint, map_error, MAX_IC_AGENT_RESPONSE_BODY_BYTES, None)
}

/// Construct an agent that verifies retained certificate signatures without imposing live age.
///
/// Callers must independently bind the certificate time to the retained observation time. This
/// agent is for local historical verification only and must not be used for live source calls.
#[cfg(feature = "certified-subnet-catalog-host")]
pub fn build_historical_certificate_agent<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
) -> Result<Agent, Error> {
    build_ic_agent_with_options(
        endpoint,
        map_error,
        MAX_IC_AGENT_RESPONSE_BODY_BYTES,
        Some(Duration::from_secs(u32::MAX.into())),
    )
}

#[cfg(any(
    feature = "ic-state-host",
    feature = "cmc-host",
    feature = "subnet-catalog-host",
    test
))]
fn build_ic_agent_with_options<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
    max_response_body_bytes: usize,
    certificate_age_limit: Option<std::time::Duration>,
) -> Result<Agent, Error> {
    let endpoint = parse_http_endpoint(endpoint).map_err(&map_error)?;
    let mut builder = ic_host_tools::agent::agent_builder(&endpoint, max_response_body_bytes);
    if let Some(certificate_age_limit) = certificate_age_limit {
        builder = builder.with_ingress_expiry(certificate_age_limit);
    }
    builder
        .build()
        .map_err(|error| map_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
    use ic_platform::agent::{AgentError, TransportError};
    use ic_platform::candid::contract::Principal;
    use std::time::Duration;

    #[test]
    fn endpoint_agent_reuses_admitted_transport_and_preserves_its_policy() {
        let source = EndpointAgent::default();
        let endpoint = "https://icp-api.io";
        let first = endpoint_agent(&source, endpoint, 64, |reason| reason).unwrap();
        let second = endpoint_agent(&source, endpoint, 64, |reason| reason).unwrap();
        assert!(std::ptr::eq(first, second));
        assert!(endpoint_agent(&source, "https://icp-api.io/", 64, |reason| reason).is_err());
        assert!(endpoint_agent(&source, "https://other.example", 64, |reason| reason).is_err());
        assert!(endpoint_agent(&source, endpoint, 65, |reason| reason).is_err());
        assert!(std::ptr::eq(
            first,
            endpoint_agent(&source, endpoint, 64, |reason| reason).unwrap()
        ));
    }

    #[test]
    fn endpoint_agent_failed_admission_leaves_transport_unselected() {
        for endpoint in [
            "invalid endpoint",
            "https://reader:secret@icp-api.io",
            "https://icp-api.io?target=other",
            "https://icp-api.io#other",
            "https://icp-api.io/\npath",
            "ftp://icp-api.io",
        ] {
            let source = EndpointAgent::default();
            assert!(endpoint_agent(&source, endpoint, 64, |reason| reason).is_err());
            assert!(endpoint_agent(&source, "https://icp-api.io", 64, |reason| reason).is_ok());
        }
    }

    #[test]
    fn constructs_agent_for_valid_endpoint_without_network_io() {
        build_ic_agent("https://icp-api.io", |reason| reason)
            .expect("valid HTTPS endpoint must construct an agent");
    }

    #[cfg(feature = "certified-subnet-catalog-host")]
    #[test]
    fn historical_builder_preserves_retained_verification_expiry() {
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let historical =
            build_historical_certificate_agent("https://icp-api.io", |reason| reason).unwrap();
        let expiry = historical
            .query(&Principal::anonymous(), "expiry_policy")
            .sign()
            .unwrap()
            .ingress_expiry
            / 1_000_000_000;
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // The SDK rounds long expiries down to a minute; signing performs no IO.
        let allowance = u64::from(u32::MAX);
        assert!((before + allowance - 60..=after + allowance).contains(&expiry));
        let live = build_ic_agent("https://icp-api.io", |reason| reason).unwrap();
        assert_eq!(historical.read_root_key(), live.read_root_key());
    }

    #[test]
    fn rejects_unclean_endpoint_before_agent_construction() {
        for endpoint in [
            "https://reader:secret@icp-api.io",
            "https://icp-api.io?target=other",
            "https://icp-api.io#other",
        ] {
            assert!(build_ic_agent(endpoint, |reason| reason).is_err());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn configured_agent_rejects_a_response_beyond_its_body_limit() {
        let server = HttpFixture::spawn(
            &[b'x'; 65],
            "application/cbor",
            HttpFixtureLimits {
                timeout: Duration::from_secs(10),
                max_header_bytes: 64 * 1024,
                max_body_bytes: 8 * 1024 * 1024,
            },
        )
        .unwrap();
        let endpoint = server.endpoint();
        let binding = EndpointAgent::default();
        let agent = endpoint_agent(&binding, endpoint, 64, |reason| reason)
            .expect("bounded endpoint agent");

        // Isolate the body limit from signature verification's concurrent read_state request.
        let error = (agent
            .query(&Principal::anonymous(), "bounded_response_test")
            .call_without_verification())
        .await
        .expect_err("oversized native response must fail");
        server.finish().expect("test replica");

        assert!(
            matches!(
                error,
                AgentError::TransportError(TransportError::Generic(_))
            ),
            "unexpected bounded-response error: {error:?}"
        );
    }
}
