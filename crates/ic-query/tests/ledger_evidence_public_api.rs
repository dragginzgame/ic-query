#![cfg(feature = "ledger-evidence-host")]

use ic_host_tools::ledger_evidence::LedgerTip;
use ic_platform::{agent::Agent, candid::contract::Principal};
use ic_query::certificate::{
    CertificateEvidenceLimits, CertificateVerificationError, authenticate_ledger_tip,
};

#[test]
fn ledger_tip_authentication_returns_host_evidence_and_preserves_byte_admission() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let result: Result<LedgerTip, CertificateVerificationError> = authenticate_ledger_tip(
        &agent,
        Principal::anonymous(),
        &[0],
        &[0],
        CertificateEvidenceLimits { bytes: 1 },
    );
    assert!(matches!(
        result,
        Err(CertificateVerificationError::BudgetExceeded)
    ));
}
