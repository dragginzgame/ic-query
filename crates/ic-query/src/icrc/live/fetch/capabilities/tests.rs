use super::*;
use crate::icrc::live::tests::{query_content, query_fixture};
use ic_platform::agent::{AgentError, RejectCode};

#[tokio::test(flavor = "current_thread")]
async fn capability_probe_retains_reject_metadata_through_the_ledger_transport() {
    for (error_code, message, expected) in [
        (
            "IC0536",
            "fixture missing method",
            IcrcCapabilityStatus::Unsupported,
        ),
        (
            "IC0502",
            "IC0536 method not found",
            IcrcCapabilityStatus::Error,
        ),
    ] {
        let body = ic_platform::ic_protocol::fixtures::query_rejection(
            &ic_platform::agent::RejectResponse {
                reject_code: RejectCode::CanisterError,
                reject_message: message.into(),
                error_code: Some(error_code.into()),
            },
            8 * 1024 * 1024,
        )
        .unwrap();
        let (server, agent) = query_fixture(&body, 8 * 1024 * 1024);
        let row = fetch_index_capability(&agent, &Principal::anonymous()).await;
        let raw = server.finish().unwrap();
        let ic_platform::ic_protocol::EnvelopeContent::Query { method_name, .. } =
            query_content(&raw)
        else {
            panic!("query content")
        };
        assert_eq!(method_name, ICRC106_GET_INDEX_PRINCIPAL_METHOD);
        assert_eq!(row.status, expected);
        let diagnostic = row.error.unwrap();
        assert!(diagnostic.contains(message));
        assert!(diagnostic.contains(error_code));
    }
}

#[test]
fn capability_failures_classify_protocol_codes_and_preserve_diagnostics() {
    for certified in [false, true] {
        for (code, error_code, message, expected) in [
            (
                RejectCode::CanisterError,
                Some("IC0536"),
                "method unavailable",
                IcrcCapabilityStatus::Unsupported,
            ),
            (
                RejectCode::CanisterError,
                Some("IC0502"),
                "IC0536: method not found",
                IcrcCapabilityStatus::Error,
            ),
            (
                RejectCode::CanisterError,
                None,
                "Canister has no query method",
                IcrcCapabilityStatus::Error,
            ),
            (
                RejectCode::CanisterReject,
                Some("IC0536"),
                "method not found",
                IcrcCapabilityStatus::Error,
            ),
        ] {
            let reject = ic_platform::agent::RejectResponse {
                reject_code: code,
                reject_message: message.to_string(),
                error_code: error_code.map(str::to_string),
            };
            let error = if certified {
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
            let original = IcrcError::AgentCall {
                method: ICRC3_GET_BLOCKS_METHOD,
                reason: error.to_string(),
            }
            .to_string();
            let row = error_capability_row(
                "ICRC-3 block history",
                ICRC3_GET_BLOCKS_METHOD,
                CapabilityQueryError::agent_call(ICRC3_GET_BLOCKS_METHOD, error.into()),
            );
            assert_eq!(row.status, expected);
            assert_eq!(row.error.as_deref(), Some(original.as_str()));
        }
    }
    let error = CapabilityQueryError::candid_decode(
        ICRC3_GET_BLOCKS_METHOD,
        "method not found IC0536".into(),
    );
    assert_eq!(
        error_capability_row("blocks", ICRC3_GET_BLOCKS_METHOD, error).status,
        IcrcCapabilityStatus::Error
    );
}
