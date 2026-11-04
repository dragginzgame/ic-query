//! Module: icrc::live::tip_certificate
//!
//! Responsibility: authenticate and validate ICRC-3 certified ledger-tip evidence.
//! Does not own: live queries, report construction, or output.
//! Boundary: accepts the wire certificate pair only after the ledger query succeeds.

use crate::{
    certification::{CertifiedDataError, authenticate_canister_hash_tree},
    icrc::model::IcrcError,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use ic_platform::certification::{HashTree, LookupResult};
use ic_platform::icrc_ledger_types::ICRC3DataCertificate;

const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";
const LAST_BLOCK_HASH_LABEL: &[u8] = b"last_block_hash";
const LAST_BLOCK_INDEX_LABEL: &[u8] = b"last_block_index";
const SHA_256_BYTES: usize = 32;

pub(super) fn verified_tip_certificate_data(
    agent: &Agent,
    ledger_canister: &Principal,
    wire: Option<ICRC3DataCertificate>,
) -> Result<Option<ICRC3DataCertificate>, IcrcError> {
    let Some(wire) = wire else {
        return Ok(None);
    };

    let hash_tree = authenticate_canister_hash_tree(
        agent,
        ledger_canister,
        &wire.certificate,
        &wire.hash_tree,
        "ledger",
    )
    .map_err(map_certified_data_error)?;
    verify_tip_leaves(&hash_tree)?;

    Ok(Some(wire))
}

fn verify_tip_leaves(hash_tree: &HashTree) -> Result<(), IcrcError> {
    let last_block_index = required_leaf(hash_tree, LAST_BLOCK_INDEX_LABEL)?;
    validate_canonical_unsigned_leb128(last_block_index)?;

    let last_block_hash = required_leaf(hash_tree, LAST_BLOCK_HASH_LABEL)?;
    if last_block_hash.len() != SHA_256_BYTES {
        return Err(invalid_tip_certificate(format!(
            "last_block_hash must contain {SHA_256_BYTES} bytes, got {}",
            last_block_hash.len()
        )));
    }

    Ok(())
}

fn map_certified_data_error(error: CertifiedDataError) -> IcrcError {
    match error {
        CertifiedDataError::Authentication { reason } => IcrcError::AgentCall {
            method: ICRC3_GET_TIP_CERTIFICATE_METHOD,
            reason: format!("tip certificate authentication failed: {reason}"),
        },
        CertifiedDataError::Invalid { reason } => invalid_tip_certificate(reason),
    }
}

fn required_leaf<'tree>(
    hash_tree: &'tree HashTree,
    label: &'static [u8],
) -> Result<&'tree [u8], IcrcError> {
    match hash_tree.lookup_path([label]) {
        LookupResult::Found(value) => Ok(value),
        LookupResult::Absent => Err(missing_tip_leaf(label, "absent")),
        LookupResult::Unknown => Err(missing_tip_leaf(label, "not proven by the partial tree")),
        LookupResult::Error => Err(missing_tip_leaf(label, "not a leaf")),
    }
}

fn missing_tip_leaf(label: &[u8], state: &str) -> IcrcError {
    invalid_tip_certificate(format!(
        "required {} leaf is {state}",
        String::from_utf8_lossy(label)
    ))
}

fn validate_canonical_unsigned_leb128(value: &[u8]) -> Result<(), IcrcError> {
    let Some((last, preceding)) = value.split_last() else {
        return Err(invalid_tip_certificate(
            "last_block_index is not an unsigned LEB128 value",
        ));
    };
    if preceding.iter().any(|byte| byte & 0x80 == 0) || last & 0x80 != 0 {
        return Err(invalid_tip_certificate(
            "last_block_index is not an unsigned LEB128 value",
        ));
    }
    if !preceding.is_empty() && *last == 0 {
        return Err(invalid_tip_certificate(
            "last_block_index is not canonically encoded as unsigned LEB128",
        ));
    }
    Ok(())
}

fn invalid_tip_certificate(reason: impl Into<String>) -> IcrcError {
    IcrcError::CandidDecode {
        message: ICRC3_GET_TIP_CERTIFICATE_METHOD,
        reason: format!("invalid certified tip evidence: {}", reason.into()),
    }
}

#[cfg(test)]
mod tests;
