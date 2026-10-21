use super::*;
use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use ic_platform::agent::{
    AgentError, HttpErrorPayload, RejectCode, RejectResponse, TransportError,
};
use serde_cbor::Value;
use std::error::Error;
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

#[test]
fn owned_error_preserves_http_payload_display_and_original_source() {
    let upstream = AgentError::HttpError(HttpErrorPayload {
        status: 504,
        content_type: Some("application/octet-stream".into()),
        content: vec![0xff, 0, 7],
    });
    let display = upstream.to_string();
    let error = QueryCallError::from(upstream);
    assert_eq!(error.failure(), QueryFailure::Http { status: 504 });
    assert_eq!(error.to_string(), display);
    let original = error
        .source()
        .unwrap()
        .downcast_ref::<AgentError>()
        .unwrap();
    let AgentError::HttpError(payload) = original else {
        panic!("lost HTTP cause")
    };
    assert_eq!(payload.content, [0xff, 0, 7]);
    assert_eq!(
        payload.content_type.as_deref(),
        Some("application/octet-stream")
    );
}

#[test]
fn rejection_projection_preserves_authentication_and_structured_code() {
    for certified in [false, true] {
        let reject = RejectResponse {
            reject_code: RejectCode::CanisterError,
            reject_message: "IC0502 in message must not override the structured code".into(),
            error_code: Some("IC0536".into()),
        };
        let upstream = if certified {
            AgentError::CertifiedReject {
                reject,
                operation: None,
            }
        } else {
            AgentError::UncertifiedReject {
                reject,
                operation: None,
            }
        };
        let display = upstream.to_string();
        let error = QueryCallError::from(upstream);
        assert_eq!(
            error.failure(),
            QueryFailure::Rejected {
                certified,
                code: 5,
                error_code: Some("IC0536")
            }
        );
        assert_eq!(error.to_string(), display);
        assert!(error.source().unwrap().is::<AgentError>());
    }
}

#[test]
fn response_timeout_and_string_lookalikes_do_not_become_connection_failures() {
    let timeout = QueryCallError::from(AgentError::TimeoutWaitingForResponse());
    assert_eq!(timeout.failure(), QueryFailure::ResponseTimeout);
    let generic = QueryCallError::from(AgentError::TransportError(TransportError::Generic(
        "connect timeout HTTP 504 IC0536".into(),
    )));
    assert_eq!(
        generic.failure(),
        QueryFailure::Transport {
            connect: false,
            timeout: false
        }
    );
    for upstream in [
        AgentError::MessageError("HTTP 502 IC0536 timeout".into()),
        AgentError::QuerySignatureVerificationFailed,
        AgentError::ResponseSizeExceededLimit(),
    ] {
        let display = upstream.to_string();
        let error = QueryCallError::from(upstream);
        assert_eq!(error.failure(), QueryFailure::Other);
        assert_eq!(error.to_string(), display);
        assert!(error.source().unwrap().is::<AgentError>());
    }
}
