//! Module: certification
//!
//! Responsibility: authenticate one canister certificate and its certified-data hash tree.
//! Does not own: authority-specific witness labels, leaf values, or public report errors.
//! Boundary: returns an authenticated tree whose digest is committed by canister certified_data.

#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
use ic_platform::agent::Agent;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
use ic_platform::candid::contract::Principal;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    all(test, any(feature = "cmc-host", feature = "icrc-host"))
))]
use ic_platform::certification::Certificate;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
use ic_platform::certification::HashTree;

/// Maximum accepted difference between caller observation and certificate time.
#[cfg(any(feature = "certified-subnet-catalog-host", feature = "ic-state-host"))]
pub const MAX_CERTIFICATE_TIME_SKEW_SECONDS: u64 = 5 * 60;

///
/// CertifiedDataError
///
/// Internal failure while authenticating a canister certified-data witness.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertifiedDataError {
    /// The certificate signature or delegation did not authenticate.
    #[cfg(any(
        feature = "certified-subnet-catalog-host",
        feature = "cmc-host",
        feature = "icrc-host"
    ))]
    Authentication {
        /// Authentication failure detail from the IC agent.
        reason: String,
    },
    /// The certificate, hash tree, or certified-data commitment was invalid.
    Invalid {
        /// Deterministic validation failure detail.
        reason: String,
    },
}

/// Authenticate a canister certificate and return its committed hash tree.
#[cfg(any(feature = "cmc-host", feature = "icrc-host"))]
pub fn authenticate_canister_hash_tree(
    agent: &Agent,
    canister: &Principal,
    encoded_certificate: &[u8],
    encoded_hash_tree: &[u8],
    certified_data_owner: &str,
) -> Result<HashTree, CertifiedDataError> {
    crate::certificate::authenticate_hash_tree(
        agent,
        *canister,
        encoded_certificate,
        encoded_hash_tree,
        crate::certificate::CertificateEvidenceLimits {
            bytes: crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES,
        },
    )
    .map_err(|error| match error {
        crate::certificate::CertificateVerificationError::Invalid(error) => {
            CertifiedDataError::Authentication {
                reason: error.to_string(),
            }
        }
        crate::certificate::CertificateVerificationError::WitnessMismatch => {
            invalid_certified_data(format!(
                "hash-tree digest does not match the {certified_data_owner} certified_data value"
            ))
        }
        error => invalid_certified_data(format!("certificate evidence is invalid: {error}")),
    })
}

/// Authenticate a decoded certificate and its authority-specific hash tree.
#[cfg(feature = "certified-subnet-catalog-host")]
pub fn authenticate_canister_tree(
    agent: &Agent,
    canister: &Principal,
    certificate: &Certificate,
    hash_tree: &HashTree,
    certified_data_owner: &str,
) -> Result<(), CertifiedDataError> {
    agent
        .verify(certificate, *canister)
        .map_err(|error| CertifiedDataError::Authentication {
            reason: error.to_string(),
        })?;
    verify_canister_tree(certificate, canister, hash_tree, certified_data_owner)
}

/// Validate that a decoded certificate commits to a decoded hash tree.
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    all(test, any(feature = "cmc-host", feature = "icrc-host"))
))]
pub fn verify_canister_tree(
    certificate: &Certificate,
    canister: &Principal,
    hash_tree: &HashTree,
    certified_data_owner: &str,
) -> Result<(), CertifiedDataError> {
    crate::certificate::verify_witness(certificate, *canister, hash_tree).map_err(|_| {
        invalid_certified_data(format!(
            "hash-tree digest does not match the {certified_data_owner} certified_data value"
        ))
    })
}

/// Validate certificate time against a caller-supplied observation time.
#[cfg(any(feature = "certified-subnet-catalog-host", feature = "ic-state-host"))]
pub fn validate_certificate_time(
    observed_at_unix_secs: u64,
    certificate_time_nanos: u64,
) -> Result<(), CertifiedDataError> {
    let certificate_time = certificate_time_nanos / 1_000_000_000;
    let minimum = observed_at_unix_secs.saturating_sub(MAX_CERTIFICATE_TIME_SKEW_SECONDS);
    let maximum = observed_at_unix_secs.saturating_add(MAX_CERTIFICATE_TIME_SKEW_SECONDS);
    if !(minimum..=maximum).contains(&certificate_time) {
        return Err(invalid_certified_data(format!(
            "certificate time {certificate_time} is outside the accepted {MAX_CERTIFICATE_TIME_SKEW_SECONDS}-second skew around collection time {observed_at_unix_secs}"
        )));
    }
    Ok(())
}

fn invalid_certified_data(reason: impl Into<String>) -> CertifiedDataError {
    CertifiedDataError::Invalid {
        reason: reason.into(),
    }
}

#[cfg(all(test, any(feature = "cmc-host", feature = "icrc-host")))]
mod tests {
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
}
