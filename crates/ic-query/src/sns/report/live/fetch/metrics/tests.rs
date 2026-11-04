use super::*;

const GOVERNANCE_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";

#[test]
fn sns_metrics_fetch_preserves_native_governance_error() {
    // External get_metrics reply carrying a Governance error through the optional result.
    let bytes = b"\x44\x49\x44\x4c\x04\x6c\x02\x90\xc6\xc1\x96\x05\x71\xd1\x9b\xc2\x8f\x0e\x75\x6b\x01\xc5\xfe\xd2\x01\x00\x6e\x01\x6c\x01\xe2\xae\xb3\xe5\x0c\x02\x01\x03\x01\x00\x13\x6d\x65\x74\x72\x69\x63\x73\x20\x75\x6e\x61\x76\x61\x69\x6c\x61\x62\x6c\x65\x07\x00\x00\x00";
    let response = crate::candid_decode::decode_reply(bytes).expect("decode SNS metrics error");
    let error = metrics_response(GOVERNANCE_CANISTER_ID, 86_400, response)
        .expect_err("native Governance error must fail");

    assert!(matches!(
        error,
        SnsHostError::GovernanceError {
            method,
            error_type: 7,
            message,
        } if method == SnsCanisterMethod::GetMetrics.as_str()
            && message == "metrics unavailable"
    ));
}

#[test]
fn sns_metrics_fetch_rejects_missing_native_result() {
    let error = metrics_response(
        GOVERNANCE_CANISTER_ID,
        86_400,
        GetMetricsResponse {
            get_metrics_result: None,
        },
    )
    .expect_err("missing Governance result must fail");

    assert!(matches!(
        error,
        SnsHostError::MissingGovernanceResult {
            method,
        } if method == SnsCanisterMethod::GetMetrics.as_str()
    ));
}
