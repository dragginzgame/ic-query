//! Bounded certificate authentication and certified-data witness admission.
//!
//! Callers own trust-root selection, observation policy and application leaf meanings.

#[cfg(test)]
mod tests;

use ic_host_tools::certificate::{
    DecodeError, DecodeLimits, ParseError, decode_certificate, decode_hash_tree,
};
#[cfg(feature = "ledger-evidence-host")]
use ic_host_tools::ledger_evidence::LedgerTip;
use ic_platform::agent::{Agent, AgentError};
use ic_platform::candid::contract::Principal;
use ic_platform::certification::{Certificate, HashTree, LookupResult};
use thiserror::Error;

///
/// CertificateEvidenceLimits
///
/// Caller-selected total wire-byte allowance for one evidence pair.
/// CBOR nesting follows Host's IC Agent codec and its built-in recursion limit.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateEvidenceLimits {
    /// Maximum combined wire bytes of the certificate and witness.
    pub bytes: usize,
}

///
/// CertificateVerificationError
///
/// Typed evidence refusal; it never establishes an application's payment outcome.
///

#[derive(Debug, Error)]
pub enum CertificateVerificationError {
    /// The certificate or combined evidence pair exceeds its wire-byte budget.
    #[error("certificate evidence exceeds its byte budget")]
    BudgetExceeded,
    /// The encoded evidence contains trailing bytes after the decoded value.
    #[error("certificate evidence encoding is invalid")]
    Malformed,
    /// CBOR decoding or bounded nesting admission failed.
    #[error("certificate CBOR decoding failed: {0}")]
    Decode(#[source] ParseError),
    /// The Agent rejected certificate authentication under its selected policy.
    #[error("certificate authentication failed")]
    Invalid(#[source] AgentError),
    /// The witness digest cannot be matched to the canister's certified-data commitment.
    #[error("witness is not committed by the certificate")]
    WitnessMismatch,
    /// The certified tip leaves are missing, malformed, noncanonical or out of range.
    #[error("certified ledger tip is missing, malformed or out of range")]
    LedgerTipMalformed,
}

/// Authenticate exact bounded bytes using the caller's Agent trust-root policy.
/// CBOR nesting follows Host's IC Agent codec and its built-in recursion limit.
///
/// # Errors
/// Refuses oversized/malformed bytes or Agent signature, delegation and time failures.
pub fn authenticate(
    agent: &Agent,
    canister: Principal,
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<Certificate, CertificateVerificationError> {
    let certificate = decode_certificate(
        bytes,
        DecodeLimits {
            bytes: maximum_bytes,
        },
    )
    .map_err(decode_error)?;
    agent
        .verify(&certificate, canister)
        .map_err(CertificateVerificationError::Invalid)?;
    Ok(certificate)
}

fn decode_error(error: DecodeError) -> CertificateVerificationError {
    match error {
        DecodeError::ByteLimit { .. } => CertificateVerificationError::BudgetExceeded,
        DecodeError::TrailingData => CertificateVerificationError::Malformed,
        DecodeError::Cbor(source) => CertificateVerificationError::Decode(source),
    }
}

/// Authenticate a bounded certificate/witness pair and return its committed tree.
///
/// # Errors
/// Refuses total-byte admission, codec nesting, authentication and witness mismatch.
pub fn authenticate_hash_tree(
    agent: &Agent,
    canister: Principal,
    certificate: &[u8],
    witness: &[u8],
    limits: CertificateEvidenceLimits,
) -> Result<HashTree, CertificateVerificationError> {
    if certificate
        .len()
        .checked_add(witness.len())
        .is_none_or(|size| size > limits.bytes)
    {
        return Err(CertificateVerificationError::BudgetExceeded);
    }
    let certificate = authenticate(agent, canister, certificate, limits.bytes)?;
    let witness = decode_hash_tree(
        witness,
        DecodeLimits {
            bytes: limits.bytes,
        },
    )
    .map_err(decode_error)?;
    verify_witness(&certificate, canister, &witness)?;
    Ok(witness)
}

/// Check a decoded witness against a certificate's exact canister commitment.
///
/// This checks a commitment only; authenticate the certificate before trusting it.
///
/// # Errors
/// Refuses missing/unknown/non-leaf commitments and unequal digests.
pub fn verify_witness(
    certificate: &Certificate,
    canister: Principal,
    witness: &HashTree,
) -> Result<(), CertificateVerificationError> {
    match certificate.tree.lookup_path([
        b"canister".as_slice(),
        canister.as_slice(),
        b"certified_data".as_slice(),
    ]) {
        LookupResult::Found(value) if value == witness.digest() => Ok(()),
        _ => Err(CertificateVerificationError::WitnessMismatch),
    }
}

/// Authenticate bounded ICRC-3 tip bytes and decode their exact index and hash.
///
/// Requires `ledger-evidence-host`; the returned Host value is authenticated by this call.
///
/// # Errors
/// Refuses certificate/witness failures, missing leaves and noncanonical/out-of-range indexes.
#[cfg(feature = "ledger-evidence-host")]
pub fn authenticate_ledger_tip(
    agent: &Agent,
    ledger: Principal,
    certificate: &[u8],
    witness: &[u8],
    limits: CertificateEvidenceLimits,
) -> Result<LedgerTip, CertificateVerificationError> {
    let tree = authenticate_hash_tree(agent, ledger, certificate, witness, limits)?;
    ledger_tip(&tree)
}

#[cfg(feature = "ledger-evidence-host")]
fn ledger_tip(tree: &HashTree) -> Result<LedgerTip, CertificateVerificationError> {
    let hash = required_leaf(tree, b"last_block_hash")?
        .try_into()
        .map_err(|_| CertificateVerificationError::LedgerTipMalformed)?;
    let index = decode_index(required_leaf(tree, b"last_block_index")?)?;
    Ok(LedgerTip { index, hash })
}

#[cfg(feature = "ledger-evidence-host")]
fn required_leaf<'a>(
    tree: &'a HashTree,
    label: &[u8],
) -> Result<&'a [u8], CertificateVerificationError> {
    match tree.lookup_path([label]) {
        LookupResult::Found(value) => Ok(value),
        _ => Err(CertificateVerificationError::LedgerTipMalformed),
    }
}

#[cfg(feature = "ledger-evidence-host")]
fn decode_index(bytes: &[u8]) -> Result<u128, CertificateVerificationError> {
    let mut value = 0_u128;
    for (offset, byte) in bytes.iter().copied().enumerate() {
        if offset > 18 || (offset == 18 && byte > 3) {
            return Err(CertificateVerificationError::LedgerTipMalformed);
        }
        value |= u128::from(byte & 0x7f) << (offset * 7);
        if byte < 0x80 {
            if offset + 1 != bytes.len() || (offset > 0 && byte == 0) {
                return Err(CertificateVerificationError::LedgerTipMalformed);
            }
            return Ok(value);
        }
    }
    Err(CertificateVerificationError::LedgerTipMalformed)
}
