//! Local IC query transport fixtures shared by ICRC protocol tests.

use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use ic_platform::agent::Agent;
use ic_platform::ic_protocol::{EnvelopeContent, decode_envelope, fixtures};
use std::time::Duration;

pub(super) fn query_fixture(response: &[u8], max_body_bytes: usize) -> (HttpFixture, Agent) {
    let server = HttpFixture::spawn(
        response,
        "application/cbor",
        HttpFixtureLimits {
            timeout: Duration::from_secs(10),
            max_header_bytes: 64 * 1024,
            max_body_bytes,
        },
    )
    .unwrap();
    let agent = Agent::builder()
        .with_url(server.endpoint())
        .with_verify_query_signatures(false)
        .build()
        .unwrap();
    (server, agent)
}

pub(super) fn replied_query_fixture(reply: &[u8], max_body_bytes: usize) -> (HttpFixture, Agent) {
    query_fixture(
        &fixtures::query_reply(reply, max_body_bytes).unwrap(),
        max_body_bytes,
    )
}

pub(super) fn query_content(request: &[u8]) -> EnvelopeContent {
    let body_start = request
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap()
        + 4;
    decode_envelope(&request[body_start..], request.len())
        .unwrap()
        .content
        .into_owned()
}
