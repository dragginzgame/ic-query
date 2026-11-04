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
    let agent =
        endpoint_agent(&binding, endpoint, 64, |reason| reason).expect("bounded endpoint agent");

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
