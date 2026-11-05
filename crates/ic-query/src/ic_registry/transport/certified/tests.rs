use super::*;
use ic_host_tools::registry::proto::registry_mixed_hash_tree::{Fork, Labeled};
use ic_platform::certification::{Certificate, labeled_hash};

#[test]
fn canonical_and_tagged_registry_certificates_require_authentication() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let canister =
        Principal::from_text(crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID).unwrap();
    let certificate = ic_platform::certification::cbor::encode_certificate_untagged(
        &Certificate {
            tree: empty(),
            signature: vec![0; 48],
            delegation: None,
        },
        64 * 1024,
    )
    .unwrap();
    let tagged = [vec![0xd9, 0xd9, 0xf7], certificate.clone()].concat();

    for certificate in [certificate, tagged] {
        let result = authenticate_registry_response(
            &agent,
            &canister,
            RegistryCertifiedResponse {
                hash_tree: Some(node(Tree::Empty(()))),
                certificate,
            },
            GET_CERTIFIED_LATEST_VERSION_METHOD,
        );
        assert!(matches!(
            result,
            Err(RegistryFetchError::CertificateAuthentication { .. })
        ));
    }
}

#[test]
fn registry_certificate_admission_rejects_malformed_and_oversized_input() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let canister =
        Principal::from_text(crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID).unwrap();
    let unsigned = Certificate {
        tree: empty(),
        signature: vec![0; 48],
        delegation: None,
    };
    let mut trailing =
        ic_platform::certification::cbor::encode_certificate_untagged(&unsigned, 64 * 1024)
            .unwrap();
    trailing.push(0);
    let unknown =
        ic_platform::ic_protocol::fixtures::certificate_with_unknown_field(&unsigned, 4096)
            .unwrap();
    let duplicate =
        ic_platform::ic_protocol::fixtures::certificate_with_duplicate_tree(&unsigned, 4096)
            .unwrap();

    for certificate in [
        vec![0xff],
        trailing,
        unknown,
        duplicate,
        vec![0; crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES + 1],
    ] {
        let result = authenticate_registry_response(
            &agent,
            &canister,
            RegistryCertifiedResponse {
                hash_tree: Some(node(Tree::Empty(()))),
                certificate,
            },
            GET_CERTIFIED_LATEST_VERSION_METHOD,
        );
        assert!(matches!(
            result,
            Err(RegistryFetchError::InvalidCertifiedRegistry { .. })
        ));
    }
}

#[test]
fn decodes_the_official_certified_latest_version_tree_shape() {
    let tree = RegistryMixedHashTree {
        tree: Some(Tree::Fork(Box::new(Fork {
            left_tree: Some(Box::new(labeled(
                CURRENT_VERSION_LABEL,
                Tree::LeafData(vec![42]),
            ))),
            right_tree: Some(Box::new(labeled(
                b"delta",
                Tree::PrunedDigest([7_u8; SHA_256_BYTES].to_vec()),
            ))),
        }))),
    };

    let decoded = decode_mixed_hash_tree(tree).expect("valid mixed hash tree");

    assert_eq!(
        required_leb128_leaf(&decoded, CURRENT_VERSION_LABEL, "current_version")
            .expect("certified version leaf"),
        42
    );
    assert_eq!(
        decoded.digest(),
        fork(
            label(CURRENT_VERSION_LABEL.to_vec(), leaf(vec![42])),
            pruned(labeled_hash(b"delta", &[7_u8; SHA_256_BYTES])),
        )
        .digest()
    );
}

#[test]
fn rejects_missing_children_and_non_sha256_pruned_digests() {
    let missing_child = RegistryMixedHashTree {
        tree: Some(Tree::Fork(Box::new(Fork {
            left_tree: None,
            right_tree: Some(Box::new(node(Tree::Empty(())))),
        }))),
    };
    assert!(matches!(
        decode_mixed_hash_tree(missing_child),
        Err(RegistryFetchError::InvalidCertifiedRegistry { reason })
            if reason.contains("fork.left_tree")
    ));

    let short_digest = node(Tree::PrunedDigest(vec![0; SHA_256_BYTES - 1]));
    assert!(matches!(
        decode_mixed_hash_tree(short_digest),
        Err(RegistryFetchError::InvalidCertifiedRegistry { reason })
            if reason.contains("31 bytes")
    ));
}

#[test]
fn rejects_missing_and_noncanonical_version_leaves() {
    let missing = decode_mixed_hash_tree(node(Tree::Empty(()))).expect("empty tree");
    assert!(matches!(
        required_leb128_leaf(&missing, CURRENT_VERSION_LABEL, "current_version"),
        Err(RegistryFetchError::InvalidCertifiedRegistry { reason })
            if reason.contains("absent")
    ));

    for bytes in [
        vec![],
        vec![0x80],
        vec![0x80, 0x00],
        vec![0, 0],
        vec![0xff; 10],
        vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 2],
    ] {
        let invalid = decode_mixed_hash_tree(labeled(CURRENT_VERSION_LABEL, Tree::LeafData(bytes)))
            .expect("structurally valid tree");
        assert!(matches!(
            required_leb128_leaf(&invalid, CURRENT_VERSION_LABEL, "current_version"),
            Err(RegistryFetchError::InvalidCertifiedRegistry { reason })
                if reason.contains("current_version")
        ));
    }
}

fn labeled(label_value: &[u8], subtree: Tree) -> RegistryMixedHashTree {
    node(Tree::Labeled(Box::new(Labeled {
        label: label_value.to_vec(),
        subtree: Some(Box::new(node(subtree))),
    })))
}

const fn node(tree: Tree) -> RegistryMixedHashTree {
    RegistryMixedHashTree { tree: Some(tree) }
}
