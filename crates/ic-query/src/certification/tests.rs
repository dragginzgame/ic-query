use super::*;

#[test]
fn oversized_evidence_refuses_before_authentication() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let certificate = vec![0; crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES];
    let error = authenticate_canister_hash_tree(
        &agent,
        &Principal::anonymous(),
        &certificate,
        &[0],
        "fixture",
    )
    .unwrap_err();
    assert!(matches!(error, CertifiedDataError::Invalid { reason }
            if reason.contains("byte budget")));
}
