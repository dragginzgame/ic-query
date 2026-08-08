//! Module: ic_registry::transport::certified
//!
//! Responsibility: authenticate bounded Registry mixed-hash-tree responses.
//! Does not own: catalog delta replay, cache publication, or report rendering.
//! Boundary: accepts a Registry value only after certificate and witness validation.

#[cfg(test)]
use crate::leb128::encode_unsigned_u64 as encode_unsigned_leb128;
use crate::{
    certification::{CertifiedDataError, authenticate_canister_tree},
    hex::hex_bytes,
    ic_registry::{
        RegistryFetchError,
        proto::{RegistryCertifiedResponse, RegistryMixedHashTree, registry_mixed_hash_tree::Tree},
    },
    leb128::decode_canonical_unsigned_u64,
};
use candid::Principal;
use ic_agent::{
    Agent, Certificate,
    hash_tree::{HashTree, LookupResult, empty, fork, label, leaf, pruned},
};
use prost::Message;
use sha2::{Digest, Sha256};

const GET_CERTIFIED_LATEST_VERSION_METHOD: &str = "get_certified_latest_version";
pub(super) const CURRENT_VERSION_LABEL: &[u8] = b"current_version";
const MAX_MIXED_HASH_TREE_DEPTH: usize = 128;
const MAX_MIXED_HASH_TREE_NODES: usize = 65_536;
const SHA_256_BYTES: usize = 32;

///
/// CertifiedRegistryVersion
///
/// Authenticated low-level evidence returned by one certified Registry query.
///

pub(in crate::ic_registry) struct CertifiedRegistryVersion {
    /// Authenticated Registry version.
    pub(in crate::ic_registry) registry_version: u64,
    /// Certificate time in nanoseconds since the Unix epoch.
    pub(in crate::ic_registry) certificate_time_nanos: u64,
    /// SHA-256 digest of the trusted DER root key.
    pub(in crate::ic_registry) root_key_digest: String,
    /// Raw CBOR certificate as lowercase hexadecimal.
    pub(in crate::ic_registry) certificate_hex: String,
    /// Raw certificate byte count.
    pub(in crate::ic_registry) certificate_bytes: usize,
    /// Encoded protobuf witness as lowercase hexadecimal.
    pub(in crate::ic_registry) hash_tree_hex: String,
    /// Encoded protobuf witness byte count.
    pub(in crate::ic_registry) hash_tree_bytes: usize,
}

///
/// AuthenticatedRegistryResponse
///
/// Authenticated common evidence from one certified Registry response.
///

pub(super) struct AuthenticatedRegistryResponse {
    pub(super) hash_tree: HashTree<Vec<u8>>,
    pub(super) certificate_time_nanos: u64,
    pub(super) root_key_digest: String,
    pub(super) certificate_hex: String,
    pub(super) certificate_bytes: usize,
    pub(super) hash_tree_hex: String,
    pub(super) hash_tree_bytes: usize,
}

pub(in crate::ic_registry) async fn get_certified_latest_version(
    agent: &Agent,
    registry_canister: &Principal,
) -> Result<CertifiedRegistryVersion, RegistryFetchError> {
    let bytes = agent
        .query(registry_canister, GET_CERTIFIED_LATEST_VERSION_METHOD)
        .with_arg(Vec::<u8>::new())
        .call()
        .await
        .map_err(|error| RegistryFetchError::AgentCall {
            method: GET_CERTIFIED_LATEST_VERSION_METHOD,
            reason: error.to_string(),
        })?;
    let response = RegistryCertifiedResponse::decode(bytes.as_slice()).map_err(|error| {
        RegistryFetchError::ProtobufDecode {
            message: "CertifiedResponse",
            reason: error.to_string(),
        }
    })?;
    verified_certified_registry_version(agent, registry_canister, response)
}

fn verified_certified_registry_version(
    agent: &Agent,
    registry_canister: &Principal,
    response: RegistryCertifiedResponse,
) -> Result<CertifiedRegistryVersion, RegistryFetchError> {
    let authenticated = authenticate_registry_response(
        agent,
        registry_canister,
        response,
        GET_CERTIFIED_LATEST_VERSION_METHOD,
    )?;
    let registry_version = required_leb128_leaf(
        &authenticated.hash_tree,
        CURRENT_VERSION_LABEL,
        "current_version",
    )?;

    Ok(CertifiedRegistryVersion {
        registry_version,
        certificate_time_nanos: authenticated.certificate_time_nanos,
        root_key_digest: authenticated.root_key_digest,
        certificate_hex: authenticated.certificate_hex,
        certificate_bytes: authenticated.certificate_bytes,
        hash_tree_hex: authenticated.hash_tree_hex,
        hash_tree_bytes: authenticated.hash_tree_bytes,
    })
}

pub(super) fn authenticate_registry_response(
    agent: &Agent,
    registry_canister: &Principal,
    response: RegistryCertifiedResponse,
    method: &'static str,
) -> Result<AuthenticatedRegistryResponse, RegistryFetchError> {
    let raw_hash_tree = response
        .hash_tree
        .ok_or_else(|| invalid_certified_registry(format!("{method} returned no hash_tree")))?;
    let encoded_hash_tree = raw_hash_tree.encode_to_vec();
    let hash_tree = decode_mixed_hash_tree(raw_hash_tree)?;
    let certificate: Certificate =
        serde_cbor::from_slice(&response.certificate).map_err(|error| {
            invalid_certified_registry(format!("certificate CBOR is invalid: {error}"))
        })?;
    authenticate_canister_tree(
        agent,
        registry_canister,
        &certificate,
        &hash_tree,
        "Registry",
    )
    .map_err(map_certified_data_error)?;

    let certificate_time =
        ic_agent::lookup_value(&certificate, [b"time".as_slice()]).map_err(|error| {
            invalid_certified_registry(format!(
                "certificate does not prove its time value: {error}"
            ))
        })?;
    let certificate_time_nanos =
        decode_canonical_unsigned_leb128("certificate time", certificate_time)?;

    Ok(AuthenticatedRegistryResponse {
        hash_tree,
        certificate_time_nanos,
        root_key_digest: hex_bytes(&Sha256::digest(agent.read_root_key())),
        certificate_hex: hex_bytes(&response.certificate),
        certificate_bytes: response.certificate.len(),
        hash_tree_hex: hex_bytes(&encoded_hash_tree),
        hash_tree_bytes: encoded_hash_tree.len(),
    })
}

fn decode_mixed_hash_tree(
    raw: RegistryMixedHashTree,
) -> Result<HashTree<Vec<u8>>, RegistryFetchError> {
    let mut nodes = 0;
    decode_mixed_hash_tree_node(raw, 0, &mut nodes)
}

fn decode_mixed_hash_tree_node(
    raw: RegistryMixedHashTree,
    depth: usize,
    nodes: &mut usize,
) -> Result<HashTree<Vec<u8>>, RegistryFetchError> {
    if depth > MAX_MIXED_HASH_TREE_DEPTH {
        return Err(invalid_certified_registry(format!(
            "mixed hash tree exceeds the maximum depth of {MAX_MIXED_HASH_TREE_DEPTH}"
        )));
    }
    *nodes = nodes.saturating_add(1);
    if *nodes > MAX_MIXED_HASH_TREE_NODES {
        return Err(invalid_certified_registry(format!(
            "mixed hash tree exceeds the maximum node count of {MAX_MIXED_HASH_TREE_NODES}"
        )));
    }

    match raw
        .tree
        .ok_or_else(|| invalid_certified_registry("mixed hash tree node is empty"))?
    {
        Tree::Empty(()) => Ok(empty()),
        Tree::Fork(branch) => {
            let left = required_child(branch.left_tree, "fork.left_tree", depth, nodes)?;
            let right = required_child(branch.right_tree, "fork.right_tree", depth, nodes)?;
            Ok(fork(left, right))
        }
        Tree::Labeled(branch) => {
            let subtree = required_child(branch.subtree, "labeled.subtree", depth, nodes)?;
            Ok(label(branch.label, subtree))
        }
        Tree::LeafData(value) => Ok(leaf(value)),
        Tree::PrunedDigest(value) => {
            let digest: [u8; SHA_256_BYTES] = value.try_into().map_err(|value: Vec<u8>| {
                invalid_certified_registry(format!(
                    "pruned digest is {} bytes; expected {SHA_256_BYTES}",
                    value.len()
                ))
            })?;
            Ok(pruned(digest))
        }
    }
}

fn required_child(
    child: Option<Box<RegistryMixedHashTree>>,
    field: &str,
    depth: usize,
    nodes: &mut usize,
) -> Result<HashTree<Vec<u8>>, RegistryFetchError> {
    let child = child
        .ok_or_else(|| invalid_certified_registry(format!("mixed hash tree {field} is missing")))?;
    decode_mixed_hash_tree_node(*child, depth.saturating_add(1), nodes)
}

pub(super) fn required_leb128_leaf(
    hash_tree: &HashTree<Vec<u8>>,
    label: &[u8],
    field: &str,
) -> Result<u64, RegistryFetchError> {
    match hash_tree.lookup_path([label]) {
        LookupResult::Found(value) => decode_canonical_unsigned_leb128(field, value),
        LookupResult::Absent => Err(invalid_certified_registry(format!(
            "{field} leaf is absent"
        ))),
        LookupResult::Unknown => Err(invalid_certified_registry(format!(
            "{field} leaf is not proven by the partial tree"
        ))),
        LookupResult::Error => Err(invalid_certified_registry(format!(
            "{field} path does not identify a leaf"
        ))),
    }
}

fn decode_canonical_unsigned_leb128(field: &str, bytes: &[u8]) -> Result<u64, RegistryFetchError> {
    decode_canonical_unsigned_u64(field, bytes).map_err(invalid_certified_registry)
}

fn map_certified_data_error(error: CertifiedDataError) -> RegistryFetchError {
    match error {
        CertifiedDataError::Authentication { reason } => {
            RegistryFetchError::CertificateAuthentication { reason }
        }
        CertifiedDataError::Invalid { reason } => invalid_certified_registry(reason),
    }
}

pub(super) fn invalid_certified_registry(reason: impl Into<String>) -> RegistryFetchError {
    RegistryFetchError::InvalidCertifiedRegistry {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic_registry::proto::registry_mixed_hash_tree::{Fork, Labeled};
    use ic_agent::hash_tree::labeled_hash;

    #[test]
    fn decodes_the_official_certified_latest_version_tree_shape() {
        let tree = RegistryMixedHashTree {
            tree: Some(Tree::Fork(Box::new(Fork {
                left_tree: Some(Box::new(labeled(
                    CURRENT_VERSION_LABEL,
                    Tree::LeafData(encode_unsigned_leb128(42)),
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
                label(
                    CURRENT_VERSION_LABEL.to_vec(),
                    leaf(encode_unsigned_leb128(42))
                ),
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

        let noncanonical = decode_mixed_hash_tree(labeled(
            CURRENT_VERSION_LABEL,
            Tree::LeafData(vec![0x80, 0x00]),
        ))
        .expect("structurally valid tree");
        assert!(matches!(
            required_leb128_leaf(&noncanonical, CURRENT_VERSION_LABEL, "current_version"),
            Err(RegistryFetchError::InvalidCertifiedRegistry { reason })
                if reason.contains("not canonical")
        ));
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
}
