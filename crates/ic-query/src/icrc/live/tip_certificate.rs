//! Module: icrc::live::tip_certificate
//!
//! Responsibility: authenticate and validate ICRC-3 certified ledger-tip evidence.
//! Does not own: live queries, report construction, or output.
//! Boundary: accepts the wire certificate pair only after the ledger query succeeds.

use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::Icrc3DataCertificate,
        model::{IcrcError, IcrcTipCertificateData},
    },
};
use candid::Principal;
use ic_agent::{
    Agent, Certificate,
    hash_tree::{HashTree, LookupResult},
};

const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";
const LAST_BLOCK_HASH_LABEL: &[u8] = b"last_block_hash";
const LAST_BLOCK_INDEX_LABEL: &[u8] = b"last_block_index";
const SHA_256_BYTES: usize = 32;

pub(super) fn verified_tip_certificate_data(
    agent: &Agent,
    ledger_canister: &Principal,
    wire: Option<Icrc3DataCertificate>,
) -> Result<IcrcTipCertificateData, IcrcError> {
    let Some(wire) = wire else {
        return Ok(IcrcTipCertificateData {
            certificate_hex: None,
            certificate_bytes: None,
            hash_tree_hex: None,
            hash_tree_bytes: None,
        });
    };

    let certificate: Certificate = serde_cbor::from_slice(&wire.certificate).map_err(|error| {
        invalid_tip_certificate(format!("certificate CBOR is invalid: {error}"))
    })?;
    agent
        .verify(&certificate, *ledger_canister)
        .map_err(|error| IcrcError::AgentCall {
            method: ICRC3_GET_TIP_CERTIFICATE_METHOD,
            reason: format!("tip certificate authentication failed: {error}"),
        })?;
    verify_tip_witness(&certificate, ledger_canister, &wire.hash_tree)?;

    Ok(IcrcTipCertificateData {
        certificate_hex: Some(hex_bytes(&wire.certificate)),
        certificate_bytes: Some(wire.certificate.len()),
        hash_tree_hex: Some(hex_bytes(&wire.hash_tree)),
        hash_tree_bytes: Some(wire.hash_tree.len()),
    })
}

fn verify_tip_witness(
    certificate: &Certificate,
    ledger_canister: &Principal,
    encoded_hash_tree: &[u8],
) -> Result<(), IcrcError> {
    let hash_tree: HashTree<Vec<u8>> = serde_cbor::from_slice(encoded_hash_tree)
        .map_err(|error| invalid_tip_certificate(format!("hash-tree CBOR is invalid: {error}")))?;
    let certified_data_path = [
        b"canister".as_slice(),
        ledger_canister.as_slice(),
        b"certified_data".as_slice(),
    ];
    let certified_data =
        ic_agent::lookup_value(certificate, certified_data_path).map_err(|error| {
            invalid_tip_certificate(format!(
                "certificate does not prove the ledger certified_data value: {error}"
            ))
        })?;

    if certified_data != hash_tree.digest() {
        return Err(invalid_tip_certificate(
            "hash-tree digest does not match the ledger certified_data value",
        ));
    }

    let last_block_index = required_leaf(&hash_tree, LAST_BLOCK_INDEX_LABEL)?;
    validate_canonical_unsigned_leb128(last_block_index)?;

    let last_block_hash = required_leaf(&hash_tree, LAST_BLOCK_HASH_LABEL)?;
    if last_block_hash.len() != SHA_256_BYTES {
        return Err(invalid_tip_certificate(format!(
            "last_block_hash must contain {SHA_256_BYTES} bytes, got {}",
            last_block_hash.len()
        )));
    }

    Ok(())
}

fn required_leaf<'tree>(
    hash_tree: &'tree HashTree<Vec<u8>>,
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
    use ic_agent::hash_tree::{fork, label, leaf};

    const LEDGER_CANISTER_ID: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

    #[test]
    fn accepts_a_matching_certified_tip_witness() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = tip_hash_tree(vec![0xac, 0x02], vec![0x5a; SHA_256_BYTES]);
        let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());
        let encoded_hash_tree = serde_cbor::to_vec(&hash_tree).expect("encode hash tree");

        verify_tip_witness(&certificate, &ledger_canister, &encoded_hash_tree)
            .expect("valid certified tip witness");
    }

    #[test]
    fn rejects_a_hash_tree_not_committed_by_certified_data() {
        let ledger_canister =
            Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
        let hash_tree = tip_hash_tree(vec![0x01], vec![0x5a; SHA_256_BYTES]);
        let certificate = certificate_for_tip(&ledger_canister, vec![0; SHA_256_BYTES]);
        let encoded_hash_tree = serde_cbor::to_vec(&hash_tree).expect("encode hash tree");

        let error = verify_tip_witness(&certificate, &ledger_canister, &encoded_hash_tree)
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
        let encoded_hash_tree = serde_cbor::to_vec(&hash_tree).expect("encode hash tree");

        let error = verify_tip_witness(&certificate, &ledger_canister, &encoded_hash_tree)
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
            let encoded_hash_tree = serde_cbor::to_vec(&hash_tree).expect("encode hash tree");

            let error = verify_tip_witness(&certificate, &ledger_canister, &encoded_hash_tree)
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
        let wire = Icrc3DataCertificate {
            certificate: serde_cbor::to_vec(&certificate).expect("encode certificate"),
            hash_tree: serde_cbor::to_vec(&hash_tree).expect("encode hash tree"),
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
        let wire = Icrc3DataCertificate {
            certificate: vec![0xff],
            hash_tree: Vec::new(),
        };
        let agent = Agent::builder()
            .with_url("https://icp-api.io")
            .build()
            .expect("build agent");

        let error = verified_tip_certificate_data(&agent, &ledger_canister, Some(wire))
            .expect_err("malformed certificate CBOR must fail");

        assert!(matches!(
            error,
            IcrcError::CandidDecode { reason, .. }
                if reason.contains("certificate CBOR is invalid")
        ));
    }

    fn tip_hash_tree(last_block_index: Vec<u8>, last_block_hash: Vec<u8>) -> HashTree<Vec<u8>> {
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
