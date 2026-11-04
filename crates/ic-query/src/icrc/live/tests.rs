//! Local IC query transport fixtures shared by ICRC protocol tests.

use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use ic_platform::agent::Agent;
use serde_cbor::Value;
use std::{collections::BTreeMap, time::Duration};

pub(super) fn query_fixture(response: &Value, max_body_bytes: usize) -> (HttpFixture, Agent) {
    let server = HttpFixture::spawn(
        &serde_cbor::to_vec(response).unwrap(),
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

pub(super) fn replied_query_fixture(reply: Vec<u8>, max_body_bytes: usize) -> (HttpFixture, Agent) {
    query_fixture(
        &Value::Map(BTreeMap::from([
            (Value::Text("status".into()), Value::Text("replied".into())),
            (
                Value::Text("reply".into()),
                Value::Map(BTreeMap::from([(
                    Value::Text("arg".into()),
                    Value::Bytes(reply),
                )])),
            ),
        ])),
        max_body_bytes,
    )
}

pub(super) fn query_content(request: &[u8]) -> BTreeMap<Value, Value> {
    let body_start = request
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap()
        + 4;
    let Value::Map(mut envelope) = serde_cbor::from_slice(&request[body_start..]).unwrap() else {
        panic!("request envelope")
    };
    let Some(Value::Map(content)) = envelope.remove(&Value::Text("content".into())) else {
        panic!("request content")
    };
    content
}
