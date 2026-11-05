use super::*;
#[cfg(all(
    not(feature = "nns-host"),
    not(feature = "sns-host"),
    feature = "cloud-engine-host"
))]
use crate::cloud_engine::CloudEngineHostError as QueryError;
#[cfg(feature = "nns-host")]
use crate::nns::governance::NnsGovernanceError as QueryError;
#[cfg(all(not(feature = "nns-host"), feature = "sns-host"))]
use crate::sns::SnsHostError as QueryError;
#[cfg(all(
    not(feature = "nns-host"),
    not(feature = "sns-host"),
    not(feature = "cloud-engine-host"),
    feature = "cmc-host"
))]
use crate::system::cmc::CmcHostError as QueryError;
use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use std::time::Duration;

#[tokio::test(flavor = "current_thread")]
async fn query_preserves_argument_lists_reply_bytes_and_family_error_labels() {
    for (argument, response_message) in [(Some(7_u64), "FixtureResponse"), (None, "EmptyResponse")]
    {
        for (reply, expected) in [
            (
                ic_platform::candid::contract::encode_one(42_u64).unwrap(),
                Some(42),
            ),
            (vec![0xff], None),
        ] {
            let body = ic_platform::ic_protocol::fixtures::query_reply(&reply, 64 * 1024).unwrap();
            let server = HttpFixture::spawn(
                &body,
                "application/cbor",
                HttpFixtureLimits {
                    timeout: Duration::from_secs(10),
                    max_header_bytes: 64 * 1024,
                    max_body_bytes: 64 * 1024,
                },
            )
            .unwrap();
            let endpoint = server.endpoint();
            let agent = Agent::builder()
                .with_url(endpoint)
                .with_verify_query_signatures(false)
                .build()
                .unwrap();
            let encoded_argument = argument
                .map_or_else(
                    || encode_request::<_, QueryError>((), "()"),
                    |value| encode_request((value,), "FixtureRequest"),
                )
                .unwrap();
            let result: Result<u64, QueryError> = query_encoded(
                &agent,
                &Principal::anonymous(),
                "fixture_query",
                response_message,
                encoded_argument.clone(),
            )
            .await;
            match expected {
                Some(expected) => assert_eq!(result.unwrap(), expected),
                None => assert!(matches!(
                    result,
                    Err(QueryError::CandidDecode {
                        message,
                        ..
                    }) if message == response_message
                )),
            }
            let raw = server.finish().unwrap();
            let body_start = raw
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .unwrap()
                + 4;
            assert!(raw[body_start..].starts_with(&[0xd9, 0xd9, 0xf7]));
            let envelope =
                ic_platform::ic_protocol::decode_envelope(&raw[body_start..], 64 * 1024).unwrap();
            let ic_platform::ic_protocol::EnvelopeContent::Query {
                arg, method_name, ..
            } = envelope.content.as_ref()
            else {
                panic!("expected query content");
            };
            assert_eq!(arg, &encoded_argument);
            assert_eq!(method_name, "fixture_query");
        }
    }
}
