use super::*;
use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use serde_cbor::Value;
use std::{collections::BTreeMap, time::Duration};

#[tokio::test(flavor = "current_thread")]
async fn raw_query_preserves_dynamic_method_target_and_non_candid_bytes() {
    let reply = vec![0xff, 0, 0x80];
    let response = Value::Map(BTreeMap::from([
        (Value::Text("status".into()), Value::Text("replied".into())),
        (
            Value::Text("reply".into()),
            Value::Map(BTreeMap::from([(
                Value::Text("arg".into()),
                Value::Bytes(reply.clone()),
            )])),
        ),
    ]));
    let server = HttpFixture::spawn(
        &serde_cbor::to_vec(&response).unwrap(),
        "application/cbor",
        HttpFixtureLimits {
            timeout: Duration::from_secs(10),
            max_header_bytes: 64 * 1024,
            max_body_bytes: 64 * 1024,
        },
    )
    .unwrap();
    // Isolated unsigned response fixture only; production verification stays configured by callers.
    let agent = Agent::builder()
        .with_url(server.endpoint())
        .with_verify_query_signatures(false)
        .build()
        .unwrap();
    let method = String::from("dynamic_archive_callback");
    let argument = vec![0, 0xff, 7];
    let canister = ExternalPrincipal::from_slice(&[1, 2, 3]);
    assert_eq!(
        query(&agent, &canister, &method, argument.clone())
            .await
            .unwrap(),
        reply
    );
    let raw = server.finish().unwrap();
    let start = raw
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap()
        + 4;
    let Value::Map(envelope) = serde_cbor::from_slice::<Value>(&raw[start..]).unwrap() else {
        panic!("expected request envelope");
    };
    let Value::Map(content) = &envelope[&Value::Text("content".into())] else {
        panic!("expected request content");
    };
    assert_eq!(content[&Value::Text("arg".into())], Value::Bytes(argument));
    assert_eq!(
        content[&Value::Text("method_name".into())],
        Value::Text(method)
    );
    assert_eq!(
        content[&Value::Text("canister_id".into())],
        Value::Bytes(canister.as_slice().to_vec())
    );
}
