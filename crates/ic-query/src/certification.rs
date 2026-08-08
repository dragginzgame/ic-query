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
use candid::Principal;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
use ic_agent::{Agent, Certificate, hash_tree::HashTree};

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
) -> Result<HashTree<Vec<u8>>, CertifiedDataError> {
    let certificate: Certificate = serde_cbor::from_slice(encoded_certificate)
        .map_err(|error| invalid_certified_data(format!("certificate CBOR is invalid: {error}")))?;
    let hash_tree: HashTree<Vec<u8>> = serde_cbor::from_slice(encoded_hash_tree)
        .map_err(|error| invalid_certified_data(format!("hash-tree CBOR is invalid: {error}")))?;
    authenticate_canister_tree(
        agent,
        canister,
        &certificate,
        &hash_tree,
        certified_data_owner,
    )?;
    Ok(hash_tree)
}

/// Authenticate a decoded certificate and its authority-specific hash tree.
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
pub fn authenticate_canister_tree(
    agent: &Agent,
    canister: &Principal,
    certificate: &Certificate,
    hash_tree: &HashTree<Vec<u8>>,
    certified_data_owner: &str,
) -> Result<(), CertifiedDataError> {
    agent
        .verify(certificate, *canister)
        .map_err(|error| CertifiedDataError::Authentication {
            reason: error.to_string(),
        })?;
    verify_canister_tree(certificate, canister, hash_tree, certified_data_owner)
}

/// Validate that a decoded certificate commits to the supplied hash tree.
#[cfg(all(test, any(feature = "cmc-host", feature = "icrc-host")))]
pub fn verify_canister_hash_tree(
    certificate: &Certificate,
    canister: &Principal,
    encoded_hash_tree: &[u8],
    certified_data_owner: &str,
) -> Result<HashTree<Vec<u8>>, CertifiedDataError> {
    let hash_tree: HashTree<Vec<u8>> = serde_cbor::from_slice(encoded_hash_tree)
        .map_err(|error| invalid_certified_data(format!("hash-tree CBOR is invalid: {error}")))?;
    verify_canister_tree(certificate, canister, &hash_tree, certified_data_owner)?;
    Ok(hash_tree)
}

/// Validate that a decoded certificate commits to a decoded hash tree.
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "cmc-host",
    feature = "icrc-host"
))]
pub fn verify_canister_tree(
    certificate: &Certificate,
    canister: &Principal,
    hash_tree: &HashTree<Vec<u8>>,
    certified_data_owner: &str,
) -> Result<(), CertifiedDataError> {
    let certified_data_path = [
        b"canister".as_slice(),
        canister.as_slice(),
        b"certified_data".as_slice(),
    ];
    let certified_data =
        ic_agent::lookup_value(certificate, certified_data_path).map_err(|error| {
            invalid_certified_data(format!(
                "certificate does not prove the {certified_data_owner} certified_data value: {error}"
            ))
        })?;

    if certified_data != hash_tree.digest() {
        return Err(invalid_certified_data(format!(
            "hash-tree digest does not match the {certified_data_owner} certified_data value"
        )));
    }

    Ok(())
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
