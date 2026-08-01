//! Module: certification
//!
//! Responsibility: authenticate one canister certificate and its certified-data hash tree.
//! Does not own: authority-specific witness labels, leaf values, or public report errors.
//! Boundary: returns an authenticated tree whose digest is committed by canister certified_data.

use candid::Principal;
use ic_agent::{Agent, Certificate, hash_tree::HashTree};

///
/// CertifiedDataError
///
/// Internal failure while authenticating a canister certified-data witness.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertifiedDataError {
    /// The certificate signature or delegation did not authenticate.
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
pub fn authenticate_canister_hash_tree(
    agent: &Agent,
    canister: &Principal,
    encoded_certificate: &[u8],
    encoded_hash_tree: &[u8],
    certified_data_owner: &str,
) -> Result<HashTree<Vec<u8>>, CertifiedDataError> {
    let certificate: Certificate = serde_cbor::from_slice(encoded_certificate)
        .map_err(|error| invalid_certified_data(format!("certificate CBOR is invalid: {error}")))?;
    agent
        .verify(&certificate, *canister)
        .map_err(|error| CertifiedDataError::Authentication {
            reason: error.to_string(),
        })?;
    verify_canister_hash_tree(
        &certificate,
        canister,
        encoded_hash_tree,
        certified_data_owner,
    )
}

/// Validate that a decoded certificate commits to the supplied hash tree.
pub fn verify_canister_hash_tree(
    certificate: &Certificate,
    canister: &Principal,
    encoded_hash_tree: &[u8],
    certified_data_owner: &str,
) -> Result<HashTree<Vec<u8>>, CertifiedDataError> {
    let hash_tree: HashTree<Vec<u8>> = serde_cbor::from_slice(encoded_hash_tree)
        .map_err(|error| invalid_certified_data(format!("hash-tree CBOR is invalid: {error}")))?;
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

    Ok(hash_tree)
}

fn invalid_certified_data(reason: impl Into<String>) -> CertifiedDataError {
    CertifiedDataError::Invalid {
        reason: reason.into(),
    }
}
