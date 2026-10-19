//! Module: agent
//!
//! Responsibility: validate endpoints before constructing response-bounded IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use crate::http_endpoint::parse_http_endpoint;
use ic_platform::agent::Agent;
#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
use std::sync::OnceLock;
#[cfg(feature = "certified-subnet-catalog-host")]
use std::time::Duration;

/// Maximum response-body size configured for every native IC agent call.
pub const MAX_IC_AGENT_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;

///
/// EndpointAgent
///
/// One lazily built, endpoint-bound agent reused by a live source collection.
///

#[derive(Clone, Default)]
#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
pub struct EndpointAgent {
    transport: OnceLock<(String, usize, Agent)>,
}

#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
impl EndpointAgent {
    pub(crate) fn get<Error>(
        &self,
        endpoint: &str,
        maximum_response_bytes: usize,
        map_error: impl Fn(String) -> Error,
    ) -> Result<&Agent, Error> {
        if self.transport.get().is_none() {
            let agent =
                build_ic_agent_with_options(endpoint, &map_error, maximum_response_bytes, None)?;
            let _ = self
                .transport
                .set((endpoint.to_owned(), maximum_response_bytes, agent));
        }
        let (selected_endpoint, selected_limit, agent) =
            self.transport.get().expect("agent initialized");
        if selected_endpoint != endpoint || *selected_limit != maximum_response_bytes {
            return Err(map_error(format!(
                "live source is bound to {selected_endpoint} with response ceiling {selected_limit}, requested {endpoint} with ceiling {maximum_response_bytes}"
            )));
        }
        Ok(agent)
    }
}

#[cfg(any(
    feature = "cloud-engine-host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host",
    test
))]
impl std::fmt::Debug for EndpointAgent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EndpointAgent")
            .field(
                "endpoint",
                &self.transport.get().map(|(endpoint, _, _)| endpoint),
            )
            .finish()
    }
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

fn build_ic_agent_with_options<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
    max_response_body_bytes: usize,
    certificate_age_limit: Option<std::time::Duration>,
) -> Result<Agent, Error> {
    parse_http_endpoint(endpoint).map_err(&map_error)?;

    let mut builder = Agent::builder()
        .with_url(endpoint)
        .with_max_response_body_size(max_response_body_bytes);
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
        let first = source.get(endpoint, 64, |reason| reason).unwrap();
        let second = source.get(endpoint, 64, |reason| reason).unwrap();
        assert!(std::ptr::eq(first, second));
        assert!(
            source
                .get("https://other.example", 64, |reason| reason)
                .is_err()
        );
        assert!(source.get(endpoint, 65, |reason| reason).is_err());
        assert!(std::ptr::eq(
            first,
            source.get(endpoint, 64, |reason| reason).unwrap()
        ));
    }

    #[test]
    fn endpoint_agent_failed_admission_leaves_transport_unselected() {
        let source = EndpointAgent::default();
        assert!(source.get("invalid endpoint", 64, |reason| reason).is_err());
        assert!(source.transport.get().is_none());
        assert!(
            source
                .get("https://icp-api.io", 64, |reason| reason)
                .is_ok()
        );
    }

    #[test]
    fn constructs_agent_for_valid_endpoint_without_network_io() {
        build_ic_agent("https://icp-api.io", |reason| reason)
            .expect("valid HTTPS endpoint must construct an agent");
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
        let agent = build_ic_agent_with_options(endpoint, |reason| reason, 64, None)
            .expect("bounded test agent");

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
