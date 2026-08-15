//! Module: agent
//!
//! Responsibility: validate endpoints before constructing response-bounded IC agents.
//! Does not own: source-specific errors, network policy, or live calls.
//! Boundary: prevents malformed endpoint text from reaching infallible parser paths in ic-agent.

use crate::http_endpoint::parse_http_endpoint;
use ic_agent::Agent;
#[cfg(feature = "certified-subnet-catalog-host")]
use std::time::Duration;

/// Maximum response-body size configured for every native IC agent call.
pub const MAX_IC_AGENT_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Validate a clean HTTP(S) endpoint and construct one response-bounded IC agent.
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

#[cfg(any(feature = "nns-host", test))]
pub fn build_ic_agent_with_response_limit<Error>(
    endpoint: &str,
    map_error: impl Fn(String) -> Error,
    max_response_body_bytes: usize,
) -> Result<Agent, Error> {
    build_ic_agent_with_options(endpoint, map_error, max_response_body_bytes, None)
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
    use crate::runtime::block_on_current_thread;
    use candid::Principal;
    use ic_agent::{AgentError, agent_error::TransportError};
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        thread,
    };

    fn read_complete_request(stream: &mut TcpStream) {
        let mut reader = BufReader::new(stream);
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let bytes_read = reader
                .read_line(&mut line)
                .expect("read test agent request");
            assert!(bytes_read > 0, "test agent request must not be empty");
            if line == "\r\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                content_length = Some(
                    value
                        .trim()
                        .parse::<usize>()
                        .expect("test agent request body length must be numeric"),
                );
            }
        }

        let mut body =
            vec![0_u8; content_length.expect("test agent request must declare its body length")];
        reader
            .read_exact(&mut body)
            .expect("read complete test agent request body");
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

    #[test]
    fn configured_agent_rejects_a_response_beyond_its_body_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test replica");
        let address = listener.local_addr().expect("test replica address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept test agent request");
            read_complete_request(&mut stream);
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
