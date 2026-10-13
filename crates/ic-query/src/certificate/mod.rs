//! Bounded certificate authentication and certified-data witness admission.
//!
//! Callers own trust-root selection, observation policy and application leaf meanings.

#[cfg(test)]
mod tests;

use candid::Principal;
use ciborium::Value;
use ic_agent::{Agent, AgentError};
use ic_certification::{
    Certificate, Delegation, HashTree, LookupResult, empty, fork, labeled, leaf as tree_leaf,
    pruned,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_bytes::ByteBuf;
use thiserror::Error;

///
/// CertificateEvidenceLimits
///
/// Caller-selected total wire-byte and CBOR nesting budgets for one evidence pair.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateEvidenceLimits {
    pub bytes: usize,
    pub depth: u8,
}

///
/// CertificateVerificationError
///
/// Typed evidence refusal; it never establishes an application's payment outcome.
///

#[derive(Debug, Error)]
pub enum CertificateVerificationError {
    #[error("certificate evidence exceeds its byte budget")]
    BudgetExceeded,
    #[error("certificate evidence encoding is invalid")]
    Malformed,
    #[error("certificate CBOR decoding failed: {0}")]
    Decode(#[source] ciborium::de::Error<std::io::Error>),
    #[error("certificate authentication failed")]
    Invalid(#[source] AgentError),
    #[error("witness is not committed by the certificate")]
    WitnessMismatch,
    #[error("certified ledger tip is missing, malformed or out of range")]
    LedgerTipMalformed,
}

/// Authenticate exact bounded bytes using the caller's Agent trust-root policy.
///
/// # Errors
/// Refuses oversized/malformed bytes or Agent signature, delegation and time failures.
pub fn authenticate(
    agent: &Agent,
    canister: Principal,
    bytes: &[u8],
    maximum_bytes: usize,
    depth: u8,
) -> Result<Certificate, CertificateVerificationError> {
    if bytes.len() > maximum_bytes {
        return Err(CertificateVerificationError::BudgetExceeded);
    }
    let wire: CertificateWire = decode(bytes, depth)?;
    let certificate = Certificate {
        tree: wire.tree.0,
        signature: wire.signature.into_vec(),
        delegation: wire.delegation,
    };
    agent
        .verify(&certificate, canister)
        .map_err(CertificateVerificationError::Invalid)?;
    Ok(certificate)
}

// ic-certification's pruned-node deserializer requires borrowed bytes, which a
// streaming CBOR reader cannot provide. Decode the bounded owned tree and map its
// exact IC tags without weakening the Agent's signature/delegation verification.
struct OwnedTree(HashTree);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertificateWire {
    tree: OwnedTree,
    signature: ByteBuf,
    #[serde(default)]
    delegation: Option<Delegation>,
}

impl<'de> Deserialize<'de> for OwnedTree {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        owned_tree(Value::deserialize(deserializer)?)
            .map(Self)
            .ok_or_else(|| serde::de::Error::custom("invalid IC hash tree"))
    }
}

fn owned_tree(value: Value) -> Option<HashTree> {
    let Value::Array(values) = value else {
        return None;
    };
    let mut values = values.into_iter();
    let Value::Integer(tag) = values.next()? else {
        return None;
    };
    let bytes = |value| {
        if let Value::Bytes(bytes) = value {
            Some(bytes)
        } else {
            None
        }
    };
    let tree = match u8::try_from(tag).ok()? {
        0 => empty(),
        1 => fork(owned_tree(values.next()?)?, owned_tree(values.next()?)?),
        2 => labeled(bytes(values.next()?)?, owned_tree(values.next()?)?),
        3 => tree_leaf(bytes(values.next()?)?),
        4 => pruned(<[u8; 32]>::try_from(bytes(values.next()?)?).ok()?),
        _ => return None,
    };
    values.next().is_none().then_some(tree)
}

fn decode<T: DeserializeOwned>(
    mut bytes: &[u8],
    depth: u8,
) -> Result<T, CertificateVerificationError> {
    let value = ciborium::de::from_reader_with_recursion_limit(&mut bytes, usize::from(depth))
        .map_err(CertificateVerificationError::Decode)?;
    if !bytes.is_empty() {
        return Err(CertificateVerificationError::Malformed);
    }
    Ok(value)
}

/// Authenticate a bounded certificate/witness pair and return its committed tree.
///
/// # Errors
/// Refuses total-byte/nesting budgets, certificate authentication and witness mismatch.
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
    let certificate = authenticate(agent, canister, certificate, limits.bytes, limits.depth)?;
    let witness = decode::<OwnedTree>(witness, limits.depth)?.0;
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

///
/// CertifiedLedgerTip
///
/// Authenticated ICRC-3 last-block index/hash; it carries no transaction or payment authority.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertifiedLedgerTip {
    pub index: u128,
    pub hash: [u8; 32],
}

/// Authenticate bounded ICRC-3 tip bytes and decode their exact index and hash.
///
/// # Errors
/// Refuses certificate/witness failures, missing leaves and noncanonical/out-of-range indexes.
pub fn authenticate_ledger_tip(
    agent: &Agent,
    ledger: Principal,
    certificate: &[u8],
    witness: &[u8],
    limits: CertificateEvidenceLimits,
) -> Result<CertifiedLedgerTip, CertificateVerificationError> {
    let tree = authenticate_hash_tree(agent, ledger, certificate, witness, limits)?;
    ledger_tip(&tree)
}

fn ledger_tip(tree: &HashTree) -> Result<CertifiedLedgerTip, CertificateVerificationError> {
    let hash = required_leaf(tree, b"last_block_hash")?
        .try_into()
        .map_err(|_| CertificateVerificationError::LedgerTipMalformed)?;
    let index = decode_index(required_leaf(tree, b"last_block_index")?)?;
    Ok(CertifiedLedgerTip { index, hash })
}

fn required_leaf<'a>(
    tree: &'a HashTree,
    label: &[u8],
) -> Result<&'a [u8], CertificateVerificationError> {
    match tree.lookup_path([label]) {
        LookupResult::Found(value) => Ok(value),
        _ => Err(CertificateVerificationError::LedgerTipMalformed),
    }
}

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
