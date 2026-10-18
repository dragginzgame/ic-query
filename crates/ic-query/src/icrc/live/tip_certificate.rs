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
mod tests {
    use super::*;
    use crate::certification::verify_canister_tree;
    use ic_platform::certification::{Certificate, fork, labeled as label, leaf};

    const LEDGER_CANISTER_ID: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

    fn verify_tip_witness(
        certificate: &Certificate,
        ledger_canister: &Principal,
        hash_tree: &HashTree,
    ) -> Result<(), IcrcError> {
        verify_canister_tree(certificate, ledger_canister, hash_tree, "ledger")
            .map_err(map_certified_data_error)?;
        verify_tip_leaves(hash_tree)
    }

    #[test]
    fn accepts_a_matching_certified_tip_witness() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = tip_hash_tree(vec![0xac, 0x02], vec![0x5a; SHA_256_BYTES]);
        let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());

        verify_tip_witness(&certificate, &ledger_canister, &hash_tree)
            .expect("valid certified tip witness");
    }

    #[test]
    fn rejects_a_hash_tree_not_committed_by_certified_data() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = tip_hash_tree(vec![0x01], vec![0x5a; SHA_256_BYTES]);
        let certificate = certificate_for_tip(&ledger_canister, vec![0; SHA_256_BYTES]);

        let error = verify_tip_witness(&certificate, &ledger_canister, &hash_tree)
            .expect_err("uncommitted hash tree must fail");

        assert!(matches!(
            error,
            IcrcError::CandidDecode { reason, .. }
                if reason.contains("digest does not match")
        ));
    }

    #[test]
    fn rejects_a_missing_required_tip_leaf() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = label(
            LAST_BLOCK_HASH_LABEL.to_vec(),
            leaf(vec![0x5a; SHA_256_BYTES]),
        );
        let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());

        let error = verify_tip_witness(&certificate, &ledger_canister, &hash_tree)
            .expect_err("missing required leaf must fail");

        assert!(matches!(
            error,
            IcrcError::CandidDecode { reason, .. }
                if reason.contains("last_block_index leaf is absent")
        ));
    }

    #[test]
    fn rejects_invalid_required_tip_leaves() {
        let cases = [
            (
                Vec::new(),
                vec![0x5a; SHA_256_BYTES],
                "not an unsigned LEB128",
            ),
            (
                vec![0x80, 0x00],
                vec![0x5a; SHA_256_BYTES],
                "not canonically encoded",
            ),
            (vec![0x01], vec![0x5a; 31], "must contain 32 bytes"),
        ];

        for (last_block_index, last_block_hash, expected) in cases {
            let ledger_canister =
                Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
            let hash_tree = tip_hash_tree(last_block_index, last_block_hash);
            let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());

            let error = verify_tip_witness(&certificate, &ledger_canister, &hash_tree)
                .expect_err("invalid required leaf must fail");

            assert!(matches!(
                error,
                IcrcError::CandidDecode { reason, .. } if reason.contains(expected)
            ));
        }
    }

    #[test]
    fn authenticates_the_certificate_before_accepting_its_witness() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = tip_hash_tree(vec![0x01], vec![0x5a; SHA_256_BYTES]);
        let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());
        let wire = ICRC3DataCertificate {
            certificate: serde_cbor::to_vec(&certificate)
                .expect("encode certificate")
                .into(),
            hash_tree: serde_cbor::to_vec(&hash_tree)
                .expect("encode hash tree")
                .into(),
        };
        let agent = Agent::builder()
            .with_url("https://icp-api.io")
            .build()
            .expect("build agent");

        let error = verified_tip_certificate_data(&agent, &ledger_canister, Some(wire))
            .expect_err("unsigned certificate must fail");

        assert!(matches!(
            error,
            IcrcError::AgentCall { reason, .. }
                if reason.contains("authentication failed")
        ));
    }

    #[test]
    fn rejects_malformed_certificate_cbor_before_authentication() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let certificate = certificate_for_tip(&ledger_canister, vec![0; SHA_256_BYTES]);
        let mut trailing = serde_cbor::to_vec(&certificate).expect("encode certificate");
        trailing.push(0);
        let agent = Agent::builder()
            .with_url("https://icp-api.io")
            .build()
            .expect("build agent");

        for certificate in [vec![0xff], trailing] {
            let wire = ICRC3DataCertificate {
                certificate: certificate.into(),
                hash_tree: Vec::new().into(),
            };
            let error = verified_tip_certificate_data(&agent, &ledger_canister, Some(wire))
                .expect_err("invalid exact certificate encoding must fail");
            assert!(matches!(error, IcrcError::CandidDecode { .. }));
        }
    }

    fn tip_hash_tree(last_block_index: Vec<u8>, last_block_hash: Vec<u8>) -> HashTree {
        fork(
            label(LAST_BLOCK_HASH_LABEL.to_vec(), leaf(last_block_hash)),
            label(LAST_BLOCK_INDEX_LABEL.to_vec(), leaf(last_block_index)),
        )
    }

    fn certificate_for_tip(ledger_canister: &Principal, certified_data: Vec<u8>) -> Certificate {
        Certificate {
            tree: label(
                b"canister".to_vec(),
                label(
                    ledger_canister.as_slice().to_vec(),
                    label(b"certified_data".to_vec(), leaf(certified_data)),
                ),
            ),
            signature: Vec::new(),
            delegation: None,
        }
    }
}
