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
    let ledger_canister = Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
    let hash_tree = tip_hash_tree(vec![0xac, 0x02], vec![0x5a; SHA_256_BYTES]);
    let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());

    verify_tip_witness(&certificate, &ledger_canister, &hash_tree)
        .expect("valid certified tip witness");
}

#[test]
fn rejects_a_hash_tree_not_committed_by_certified_data() {
    let ledger_canister = Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
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
    let ledger_canister = Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
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
    let ledger_canister = Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
    let hash_tree = tip_hash_tree(vec![0x01], vec![0x5a; SHA_256_BYTES]);
    let certificate = certificate_for_tip(&ledger_canister, hash_tree.digest().to_vec());
    let wire = ICRC3DataCertificate {
        certificate: ic_platform::certification::cbor::encode_certificate_untagged(
            &certificate,
            64 * 1024,
        )
        .expect("encode certificate")
        .into(),
        hash_tree: ic_platform::certification::cbor::encode_hash_tree(&hash_tree, 64 * 1024)
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
    let ledger_canister = Principal::from_text(LEDGER_CANISTER_ID).expect("valid ledger canister");
    let certificate = certificate_for_tip(&ledger_canister, vec![0; SHA_256_BYTES]);
    let mut trailing =
        ic_platform::certification::cbor::encode_certificate_untagged(&certificate, 64 * 1024)
            .expect("encode certificate");
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
