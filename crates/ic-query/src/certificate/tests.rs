use super::*;

fn encoded_tree(tree: &HashTree) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(tree, &mut bytes).unwrap();
    bytes
}

#[test]
fn bounded_owned_tree_accepts_pruned_nodes() {
    let tree = fork(tree_leaf(vec![1]), pruned([2; 32]));
    let bytes = encoded_tree(&tree);
    let decoded = decode::<OwnedTree>(&bytes, 16).unwrap().0;
    assert_eq!(decoded.digest(), tree.digest());
}

#[test]
fn decoder_refuses_trailing_bytes_and_excessive_nesting() {
    let mut bytes = encoded_tree(&tree_leaf(vec![1]));
    bytes.push(0);
    assert!(matches!(
        decode::<OwnedTree>(&bytes, 16),
        Err(CertificateVerificationError::Malformed)
    ));
    let tree = labeled(
        b"one".to_vec(),
        labeled(b"two".to_vec(), tree_leaf(vec![1])),
    );
    assert!(matches!(
        decode::<OwnedTree>(&encoded_tree(&tree), 1),
        Err(CertificateVerificationError::Decode(_))
    ));
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
        CertificateEvidenceLimits {
            bytes: 1,
            depth: 16,
        },
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
fn tip_requires_exact_hash_and_canonical_bounded_index() {
    let tree = fork(
        labeled(b"last_block_hash".to_vec(), tree_leaf(vec![9; 32])),
        labeled(b"last_block_index".to_vec(), tree_leaf(vec![0xac, 2])),
    );
    assert_eq!(
        ledger_tip(&tree).unwrap(),
        CertifiedLedgerTip {
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
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&certificate, &mut bytes).unwrap();
    let result = authenticate_hash_tree(
        &agent,
        Principal::anonymous(),
        &bytes,
        &[0xff],
        CertificateEvidenceLimits {
            bytes: bytes.len() + 1,
            depth: 32,
        },
    );
    assert!(matches!(
        result,
        Err(CertificateVerificationError::Invalid(_))
    ));
}
