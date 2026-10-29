use super::*;
#[cfg(feature = "ledger-evidence-host")]
use ic_host_tools::ledger_evidence::{BlockEvidenceLimits, BoundedLedgerBlocks, LedgerBlock};
#[cfg(feature = "ledger-evidence-host")]
use ic_platform::certification::fork;
use ic_platform::certification::{empty, labeled, leaf as tree_leaf};
#[cfg(feature = "ledger-evidence-host")]
use ic_platform::{candid::contract::Nat, icrc_ledger_types::Value};
#[cfg(feature = "ledger-evidence-host")]
use std::collections::BTreeMap;

fn encoded_tree(tree: &HashTree) -> Vec<u8> {
    serde_cbor::to_vec(tree).unwrap()
}

#[test]
fn shared_decoder_failures_keep_query_categories_and_original_cause() {
    assert!(matches!(
        decode_error(DecodeError::ByteLimit {
            actual: 2,
            limit: 1
        }),
        CertificateVerificationError::BudgetExceeded
    ));
    let mut trailing = encoded_tree(&tree_leaf(vec![1]));
    trailing.push(0);
    let mut nested = tree_leaf(vec![1]);
    for _ in 0..200 {
        nested = labeled(b"nested".to_vec(), nested);
    }
    for (bytes, trailing_data) in [(trailing, true), (encoded_tree(&nested), false)] {
        let error = decode_hash_tree(&bytes, DecodeLimits { bytes: bytes.len() })
            .map_err(decode_error)
            .unwrap_err();
        if trailing_data {
            assert!(matches!(error, CertificateVerificationError::Malformed));
        } else {
            let CertificateVerificationError::Decode(source) = error else {
                panic!("expected typed CBOR parse failure");
            };
            assert!(std::error::Error::source(&source).is_some());
        }
    }
}

#[test]
fn certificate_codec_controls_nesting_before_authentication() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    for nesting in [32, 200] {
        let mut tree = empty();
        for _ in 0..nesting {
            tree = labeled(b"nested".to_vec(), tree);
        }
        let certificate = Certificate {
            tree,
            signature: Vec::new(),
            delegation: None,
        };
        let bytes = serde_cbor::to_vec(&certificate).unwrap();
        let error = authenticate(&agent, Principal::anonymous(), &bytes, bytes.len()).unwrap_err();
        if nesting == 32 {
            assert!(matches!(error, CertificateVerificationError::Invalid(_)));
        } else {
            let CertificateVerificationError::Decode(source) = error else {
                panic!("expected nesting refusal before authentication");
            };
            assert!(std::error::Error::source(&source).is_some());
        }
    }
}

#[test]
fn byte_budget_refuses_before_certificate_authentication() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let result = authenticate_hash_tree(
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

#[test]
fn witness_requires_the_exact_canister_commitment() {
    let canister = Principal::from_slice(&[1, 2, 3]);
    let witness = tree_leaf(vec![9]);
    let certificate = Certificate {
        tree: labeled(
            b"canister".to_vec(),
            labeled(
                canister.as_slice().to_vec(),
                labeled(
                    b"certified_data".to_vec(),
                    tree_leaf(witness.digest().to_vec()),
                ),
            ),
        ),
        signature: Vec::new(),
        delegation: None,
    };
    verify_witness(&certificate, canister, &witness).unwrap();
    assert!(matches!(
        verify_witness(&certificate, Principal::anonymous(), &witness),
        Err(CertificateVerificationError::WitnessMismatch)
    ));
    assert!(matches!(
        verify_witness(&certificate, canister, &tree_leaf(vec![8])),
        Err(CertificateVerificationError::WitnessMismatch)
    ));
}

#[test]
#[cfg(feature = "ledger-evidence-host")]
fn tip_requires_exact_hash_and_canonical_bounded_index() {
    let tree = fork(
        labeled(b"last_block_hash".to_vec(), tree_leaf(vec![9; 32])),
        labeled(b"last_block_index".to_vec(), tree_leaf(vec![0xac, 2])),
    );
    assert_eq!(
        ledger_tip(&tree).unwrap(),
        LedgerTip {
            index: 300,
            hash: [9; 32]
        }
    );
    assert!(matches!(
        ledger_tip(&tree_leaf(vec![9; 32])),
        Err(CertificateVerificationError::LedgerTipMalformed)
    ));
    for bytes in [
        vec![],
        vec![0x80, 0],
        vec![0x80],
        vec![1, 0],
        vec![0xff; 19],
    ] {
        assert!(matches!(
            decode_index(&bytes),
            Err(CertificateVerificationError::LedgerTipMalformed)
        ));
    }
    let mut maximum = vec![0xff; 18];
    maximum.push(3);
    assert_eq!(decode_index(&maximum).unwrap(), u128::MAX);
}

#[test]
#[cfg(feature = "ledger-evidence-host")]
fn admitted_tip_connects_directly_to_the_host_chain_verifier() {
    let block = Value::Map(BTreeMap::new());
    let tree = fork(
        labeled(
            b"last_block_hash".to_vec(),
            tree_leaf(block.hash().to_vec()),
        ),
        labeled(b"last_block_index".to_vec(), tree_leaf(vec![0])),
    );
    let tip = ledger_tip(&tree).unwrap();
    let blocks = [LedgerBlock {
        id: Nat::from(0_u8),
        block,
    }];
    let admitted = BoundedLedgerBlocks::admit(
        &blocks,
        BlockEvidenceLimits {
            blocks: 1,
            nodes: 1,
            bytes: 0,
            depth: 1,
        },
    )
    .unwrap();
    assert_eq!(admitted.verify_chain(&tip, 0).unwrap(), &blocks[0].block);
}

#[test]
fn certificate_authentication_precedes_witness_admission() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let certificate = Certificate {
        tree: empty(),
        signature: Vec::new(),
        delegation: None,
    };
    let bytes = serde_cbor::to_vec(&certificate).unwrap();
    let result = authenticate_hash_tree(
        &agent,
        Principal::anonymous(),
        &bytes,
        &[0xff],
        CertificateEvidenceLimits {
            bytes: bytes.len() + 1,
        },
    );
    assert!(matches!(
        result,
        Err(CertificateVerificationError::Invalid(_))
    ));
}
