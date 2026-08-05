//! Module: agent
//!
//! Responsibility: validate live source endpoints before constructing IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use crate::http_endpoint::parse_http_endpoint;
use ic_agent::Agent;

/// Maximum response-body size configured for every native IC agent call.
pub const MAX_IC_AGENT_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Validate a clean HTTP(S) endpoint and construct one response-bounded IC agent.
pub fn build_ic_agent<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
) -> Result<Agent, Error> {
    build_ic_agent_with_response_limit(endpoint, map_error, MAX_IC_AGENT_RESPONSE_BODY_BYTES)
}

fn build_ic_agent_with_response_limit<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
    max_response_body_bytes: usize,
) -> Result<Agent, Error> {
    parse_http_endpoint(endpoint).map_err(&map_error)?;

    Agent::builder()
        .with_url(endpoint)
        .with_max_response_body_size(max_response_body_bytes)
        .build()
        .map_err(|error| map_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::block_on_current_thread;
    use candid::Principal;
    use ic_agent::{AgentError, agent_error::TransportError};
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

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

    #[test]
    fn configured_agent_rejects_a_response_beyond_its_body_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test replica");
        let address = listener.local_addr().expect("test replica address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept test agent request");
            let mut request = [0_u8; 2_048];
            let bytes_read = stream.read(&mut request).expect("read test agent request");
            assert!(bytes_read > 0, "test agent request must not be empty");
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: 65\r\nConnection: close\r\n\r\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                )
                .expect("write oversized test agent response");
        });
        let endpoint = format!("http://{address}");
        let agent = build_ic_agent_with_response_limit(&endpoint, |reason| reason, 64)
            .expect("bounded test agent");

        let error = block_on_current_thread(
            agent
                .query(&Principal::anonymous(), "bounded_response_test")
                .call(),
        )
        .expect("test query runtime")
        .expect_err("oversized native response must fail");
        server.join().expect("test replica");

        assert!(
            matches!(
                error,
                AgentError::TransportError(TransportError::Generic(_))
            ),
            "unexpected bounded-response error: {error:?}"
        );
    }
}
