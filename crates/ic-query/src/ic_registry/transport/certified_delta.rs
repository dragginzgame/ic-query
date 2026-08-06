//! Module: ic_registry::transport::certified_delta
//!
//! Responsibility: authenticate and validate one bounded Registry delta batch.
//! Does not own: historical replay, cache publication, or catalog assurance.
//! Boundary: returns only contiguous mutations with complete, bounded value content.

use super::certified::{
    CURRENT_VERSION_LABEL, authenticate_registry_response, invalid_certified_registry,
    required_leb128_leaf,
};
use super::chunk::{
    RegistryChunkBudget, RegistryChunkLimits, get_large_registry_value, validated_chunk_hashes,
};
use crate::{
    hex::hex_bytes,
    ic_registry::{
        CertifiedRegistryDeltaBatch, CertifiedRegistryDeltaVersion, CertifiedRegistryMutation,
        CertifiedRegistryPrecondition, CertifiedRegistryValueEncoding, RegistryFetchError,
        proto::{
            HighCapacityRegistryAtomicMutateRequest, HighCapacityRegistryMutation,
            RegistryCertifiedResponse, RegistryGetChangesSinceRequest, RegistryMutationType,
            high_capacity_registry_mutation::Content,
        },
    },
};
use candid::Principal;
use ic_agent::{
    Agent,
    hash_tree::{HashTree, LookupResult, SubtreeLookupResult},
};
use prost::Message;
use std::collections::BTreeSet;

const GET_CERTIFIED_CHANGES_SINCE_METHOD: &str = "get_certified_changes_since";
const DELTA_LABEL: &[u8] = b"delta";

pub const MAX_CERTIFIED_DELTA_VERSIONS: usize = 1_000;
pub const MAX_CERTIFIED_DELTA_MUTATIONS: usize = 65_536;
pub const MAX_CERTIFIED_DELTA_PRECONDITIONS: usize = 65_536;
pub const MAX_CERTIFIED_DELTA_KEY_BYTES: usize = 4_096;
pub const MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES: usize = 2 * 1_024 * 1_024;

pub(in crate::ic_registry) async fn get_certified_changes_since(
    agent: &Agent,
    registry_canister: &Principal,
    requested_version: u64,
) -> Result<CertifiedRegistryDeltaBatch, RegistryFetchError> {
    let request = RegistryGetChangesSinceRequest {
        version: requested_version,
    };
    let bytes = agent
        .query(registry_canister, GET_CERTIFIED_CHANGES_SINCE_METHOD)
        .with_arg(request.encode_to_vec())
        .call()
        .await
        .map_err(|error| RegistryFetchError::AgentCall {
            method: GET_CERTIFIED_CHANGES_SINCE_METHOD,
            reason: error.to_string(),
        })?;
    let response_bytes = bytes.len();
    let response = RegistryCertifiedResponse::decode(bytes.as_slice()).map_err(|error| {
        RegistryFetchError::ProtobufDecode {
            message: "CertifiedResponse",
            reason: error.to_string(),
        }
    })?;
    let authenticated = authenticate_registry_response(
        agent,
        registry_canister,
        response,
        GET_CERTIFIED_CHANGES_SINCE_METHOD,
    )?;
    let certified_latest_version = required_leb128_leaf(
        &authenticated.hash_tree,
        CURRENT_VERSION_LABEL,
        "current_version",
    )?;
    let mut validated = validate_delta_tree(
        &authenticated.hash_tree,
        requested_version,
        certified_latest_version,
    )?;
    let mut chunk_budget = RegistryChunkBudget::new(
        RegistryChunkLimits::certified_delta(),
        validated.inline_value_bytes,
    )?;
    complete_chunked_values(
        agent,
        registry_canister,
        &mut validated.versions,
        &mut chunk_budget,
    )
    .await?;
    if chunk_budget.reference_count() != validated.chunk_reference_count {
        return Err(invalid_certified_registry(
            "completed chunk reference count does not match the certified delta tree",
        ));
    }
    let value_bytes = chunk_budget.reconstructed_value_bytes();
    let chunk_value_bytes = value_bytes
        .checked_sub(validated.inline_value_bytes)
        .ok_or_else(|| invalid_certified_registry("completed value byte accounting underflows"))?;

    Ok(CertifiedRegistryDeltaBatch {
        requested_version,
        certified_latest_version,
        versions: validated.versions,
        mutation_count: validated.mutation_count,
        precondition_count: validated.precondition_count,
        inline_value_bytes: validated.inline_value_bytes,
        chunk_value_bytes,
        value_bytes,
        chunk_reference_count: chunk_budget.reference_count(),
        chunk_query_call_count: chunk_budget.query_call_count(),
        chunk_response_bytes: chunk_budget.response_bytes(),
        more_available: validated.more_available,
        certified_response_bytes: response_bytes,
        certificate_time_nanos: authenticated.certificate_time_nanos,
        root_key_digest: authenticated.root_key_digest,
        certificate_hex: authenticated.certificate_hex,
        certificate_bytes: authenticated.certificate_bytes,
        hash_tree_hex: authenticated.hash_tree_hex,
        hash_tree_bytes: authenticated.hash_tree_bytes,
    })
}

#[derive(Debug)]
struct ValidatedDeltaTree {
    versions: Vec<CertifiedRegistryDeltaVersion>,
    mutation_count: usize,
    precondition_count: usize,
    inline_value_bytes: usize,
    chunk_reference_count: usize,
    more_available: bool,
}

#[derive(Default)]
struct DeltaCounters {
    mutation_count: usize,
    precondition_count: usize,
    inline_value_bytes: usize,
    chunk_reference_count: usize,
}

struct VisibleDeltaTree {
    tree: HashTree<Vec<u8>>,
    labels: Vec<Vec<u8>>,
}

fn validate_delta_tree(
    tree: &HashTree<Vec<u8>>,
    requested_version: u64,
    certified_latest_version: u64,
) -> Result<ValidatedDeltaTree, RegistryFetchError> {
    let visible = visible_delta_labels(tree, requested_version, certified_latest_version)?;
    if visible.labels.is_empty() {
        return Ok(ValidatedDeltaTree {
            versions: Vec::new(),
            mutation_count: 0,
            precondition_count: 0,
            inline_value_bytes: 0,
            chunk_reference_count: 0,
            more_available: false,
        });
    }

    let mut expected_version = requested_version.checked_add(1).ok_or_else(|| {
        invalid_certified_registry("requested version cannot advance without overflowing u64")
    })?;
    let mut versions = Vec::with_capacity(visible.labels.len());
    let mut counters = DeltaCounters::default();

    for label in visible.labels {
        let version = decode_version_label(&label)?;
        if version != expected_version {
            return Err(invalid_certified_registry(format!(
                "delta version sequence expected {expected_version}, found {version}"
            )));
        }
        if version > certified_latest_version {
            return Err(invalid_certified_registry(format!(
                "delta version {version} exceeds certified latest version {certified_latest_version}"
            )));
        }
        let bytes = match visible.tree.lookup_path([&label]) {
            LookupResult::Found(bytes) => bytes,
            LookupResult::Absent => {
                return Err(invalid_certified_registry(format!(
                    "delta version {version} leaf is absent"
                )));
            }
            LookupResult::Unknown => {
                return Err(invalid_certified_registry(format!(
                    "delta version {version} leaf is not proven"
                )));
            }
            LookupResult::Error => {
                return Err(invalid_certified_registry(format!(
                    "delta version {version} path does not identify a leaf"
                )));
            }
        };
        let atomic = HighCapacityRegistryAtomicMutateRequest::decode(bytes).map_err(|error| {
            RegistryFetchError::ProtobufDecode {
                message: "HighCapacityRegistryAtomicMutateRequest",
                reason: error.to_string(),
            }
        })?;
        let validated = validate_atomic_delta(version, atomic, &mut counters)?;
        versions.push(validated);
        expected_version = expected_version
            .checked_add(1)
            .ok_or_else(|| invalid_certified_registry("delta version sequence overflows u64"))?;
    }

    let last_version = versions.last().map_or(requested_version, |row| row.version);
    Ok(ValidatedDeltaTree {
        versions,
        mutation_count: counters.mutation_count,
        precondition_count: counters.precondition_count,
        inline_value_bytes: counters.inline_value_bytes,
        chunk_reference_count: counters.chunk_reference_count,
        more_available: last_version < certified_latest_version,
    })
}

fn visible_delta_labels(
    tree: &HashTree<Vec<u8>>,
    requested_version: u64,
    certified_latest_version: u64,
) -> Result<VisibleDeltaTree, RegistryFetchError> {
    if requested_version > certified_latest_version {
        return Err(invalid_certified_registry(format!(
            "requested version {requested_version} exceeds certified latest version {certified_latest_version}"
        )));
    }
    let delta_tree = match tree.lookup_subtree([DELTA_LABEL]) {
        SubtreeLookupResult::Found(tree) => tree,
        SubtreeLookupResult::Absent | SubtreeLookupResult::Unknown
            if requested_version == certified_latest_version =>
        {
            ic_agent::hash_tree::empty()
        }
        SubtreeLookupResult::Absent => {
            return Err(invalid_certified_registry(
                "delta subtree is absent before the certified latest version",
            ));
        }
        SubtreeLookupResult::Unknown => {
            return Err(invalid_certified_registry(
                "delta subtree is not proven before the certified latest version",
            ));
        }
    };
    let paths = delta_tree.list_paths();
    validate_visible_version_count(paths.len())?;
    if requested_version < certified_latest_version && paths.is_empty() {
        return Err(invalid_certified_registry(
            "certified delta batch is empty before the certified latest version",
        ));
    }
    if requested_version == certified_latest_version && !paths.is_empty() {
        return Err(invalid_certified_registry(
            "certified delta batch contains versions after the certified latest version",
        ));
    }
    let labels = paths
        .into_iter()
        .map(|path| {
            if path.len() != 1 {
                return Err(invalid_certified_registry(format!(
                    "delta leaf path has {} labels; expected exactly one version label",
                    path.len()
                )));
            }
            Ok(path[0].as_ref().to_vec())
        })
        .collect::<Result<Vec<_>, RegistryFetchError>>()?;
    Ok(VisibleDeltaTree {
        tree: delta_tree,
        labels,
    })
}

fn validate_visible_version_count(count: usize) -> Result<(), RegistryFetchError> {
    if count > MAX_CERTIFIED_DELTA_VERSIONS {
        return Err(invalid_certified_registry(format!(
            "certified delta batch has {count} visible versions; maximum is {MAX_CERTIFIED_DELTA_VERSIONS}"
        )));
    }
    Ok(())
}

fn decode_version_label(label: &[u8]) -> Result<u64, RegistryFetchError> {
    let bytes: [u8; 8] = label.try_into().map_err(|_| {
        invalid_certified_registry(format!(
            "delta version label is {} bytes; expected exactly 8",
            label.len()
        ))
    })?;
    Ok(u64::from_be_bytes(bytes))
}

fn validate_atomic_delta(
    version: u64,
    atomic: HighCapacityRegistryAtomicMutateRequest,
    counters: &mut DeltaCounters,
) -> Result<CertifiedRegistryDeltaVersion, RegistryFetchError> {
    if atomic.mutations.is_empty() {
        return Err(invalid_certified_registry(format!(
            "delta version {version} contains no mutations"
        )));
    }
    checked_accumulate(
        "mutation count",
        &mut counters.mutation_count,
        atomic.mutations.len(),
        MAX_CERTIFIED_DELTA_MUTATIONS,
    )?;
    checked_accumulate(
        "precondition count",
        &mut counters.precondition_count,
        atomic.preconditions.len(),
        MAX_CERTIFIED_DELTA_PRECONDITIONS,
    )?;

    if let Some(pair) = atomic
        .mutations
        .windows(2)
        .find(|pair| pair[0].key > pair[1].key)
    {
        return Err(invalid_certified_registry(format!(
            "delta version {version} mutation key {} follows {} out of canonical order",
            hex_bytes(&pair[1].key),
            hex_bytes(&pair[0].key)
        )));
    }
    let mutations = atomic
        .mutations
        .into_iter()
        .map(|mutation| validate_mutation(version, mutation, counters))
        .collect::<Result<Vec<_>, RegistryFetchError>>()?;

    let mut precondition_keys = BTreeSet::new();
    let preconditions = atomic
        .preconditions
        .into_iter()
        .map(|precondition| {
            validate_key(version, "precondition", &precondition.key)?;
            if !precondition_keys.insert(precondition.key.clone()) {
                return Err(invalid_certified_registry(format!(
                    "delta version {version} has duplicate precondition key {}",
                    hex_bytes(&precondition.key)
                )));
            }
            Ok(CertifiedRegistryPrecondition {
                key_hex: hex_bytes(&precondition.key),
                expected_version: precondition.expected_version,
            })
        })
        .collect::<Result<Vec<_>, RegistryFetchError>>()?;

    Ok(CertifiedRegistryDeltaVersion {
        version,
        timestamp_nanoseconds: atomic.timestamp_nanoseconds,
        mutations,
        preconditions,
    })
}

fn validate_mutation(
    version: u64,
    mutation: HighCapacityRegistryMutation,
    counters: &mut DeltaCounters,
) -> Result<CertifiedRegistryMutation, RegistryFetchError> {
    validate_key(version, "mutation", &mutation.key)?;
    let kind = RegistryMutationType::try_from(mutation.mutation_type).map_err(|_| {
        invalid_certified_registry(format!(
            "delta version {version} has unknown mutation type {} for key {}",
            mutation.mutation_type,
            hex_bytes(&mutation.key)
        ))
    })?;
    let (value_hex, value_encoding, chunk_sha256s) = match (kind, mutation.content) {
        (RegistryMutationType::Delete, None) => {
            (None, CertifiedRegistryValueEncoding::Absent, Vec::new())
        }
        (_, Some(Content::Value(value))) => {
            checked_accumulate(
                "inline value bytes",
                &mut counters.inline_value_bytes,
                value.len(),
                MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES,
            )?;
            (
                Some(hex_bytes(&value)),
                CertifiedRegistryValueEncoding::Inline,
                Vec::new(),
            )
        }
        (_, Some(Content::LargeValueChunkKeys(chunks))) => {
            validated_chunk_hashes(&chunks.chunk_content_sha256s)?;
            checked_accumulate(
                "chunk reference count",
                &mut counters.chunk_reference_count,
                chunks.chunk_content_sha256s.len(),
                super::chunk::MAX_REGISTRY_CHUNK_REFERENCES,
            )?;
            (
                None,
                CertifiedRegistryValueEncoding::Chunked,
                chunks.chunk_content_sha256s,
            )
        }
        // Legacy RegistryMutation used a plain proto3 bytes field. Its empty
        // value is absent on the wire and therefore decodes as no oneof arm in
        // HighCapacityRegistryMutation. The official Registry transport treats
        // that representation as an empty inline value.
        (_, None) => (
            Some(String::new()),
            CertifiedRegistryValueEncoding::Inline,
            Vec::new(),
        ),
    };
    Ok(CertifiedRegistryMutation {
        mutation_type: mutation.mutation_type,
        key_hex: hex_bytes(&mutation.key),
        value_hex,
        value_encoding,
        chunk_sha256s,
    })
}

async fn complete_chunked_values(
    agent: &Agent,
    registry_canister: &Principal,
    versions: &mut [CertifiedRegistryDeltaVersion],
    budget: &mut RegistryChunkBudget,
) -> Result<(), RegistryFetchError> {
    for version in versions {
        for mutation in &mut version.mutations {
            if mutation.value_encoding != CertifiedRegistryValueEncoding::Chunked {
                continue;
            }
            let value = get_large_registry_value(
                agent,
                registry_canister,
                &mutation.chunk_sha256s,
                None,
                budget,
            )
            .await?;
            mutation.value_hex = Some(hex_bytes(&value));
        }
    }
    Ok(())
}

fn validate_key(version: u64, relation: &str, key: &[u8]) -> Result<(), RegistryFetchError> {
    if key.is_empty() {
        return Err(invalid_certified_registry(format!(
            "delta version {version} {relation} key is empty"
        )));
    }
    if key.len() > MAX_CERTIFIED_DELTA_KEY_BYTES {
        return Err(invalid_certified_registry(format!(
            "delta version {version} {relation} key is {} bytes; maximum is {MAX_CERTIFIED_DELTA_KEY_BYTES}",
            key.len()
        )));
    }
    Ok(())
}

fn checked_accumulate(
    field: &str,
    total: &mut usize,
    increment: usize,
    maximum: usize,
) -> Result<(), RegistryFetchError> {
    *total = total.checked_add(increment).ok_or_else(|| {
        invalid_certified_registry(format!("certified delta {field} exceeds usize"))
    })?;
    if *total > maximum {
        return Err(invalid_certified_registry(format!(
            "certified delta {field} is {total}; maximum is {maximum}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic_registry::proto::{
        HighCapacityRegistryMutation, RegistryPrecondition, high_capacity_registry_mutation,
    };
    use ic_agent::hash_tree::{empty, fork, label, leaf, pruned};

    #[derive(Clone, PartialEq, prost::Message)]
    struct LegacyRegistryMutation {
        #[prost(int32, tag = "1")]
        mutation_type: i32,
        #[prost(bytes = "vec", tag = "2")]
        key: Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        value: Vec<u8>,
    }

    #[test]
    fn validates_contiguous_ordered_mutations_and_more_available() {
        let tree = registry_tree(
            5,
            vec![
                delta_leaf(3, atomic(vec![upsert(b"alpha", b"one")], vec![])),
                delta_leaf(
                    4,
                    atomic(
                        vec![delete(b"alpha")],
                        vec![RegistryPrecondition {
                            key: b"alpha".to_vec(),
                            expected_version: 3,
                        }],
                    ),
                ),
            ],
        );

        let batch = validate_delta_tree(&tree, 2, 5).expect("valid partial batch");

        assert_eq!(batch.versions.len(), 2);
        assert_eq!(batch.versions[0].version, 3);
        assert_eq!(
            batch.versions[0].mutations[0].value_hex.as_deref(),
            Some("6f6e65")
        );
        assert_eq!(batch.mutation_count, 2);
        assert_eq!(batch.precondition_count, 1);
        assert_eq!(batch.inline_value_bytes, 3);
        assert!(batch.more_available);
    }

    #[test]
    fn treats_legacy_absent_empty_value_as_inline_empty_content() {
        let legacy = LegacyRegistryMutation {
            mutation_type: RegistryMutationType::Upsert as i32,
            key: b"_".to_vec(),
            value: Vec::new(),
        };
        let decoded = HighCapacityRegistryMutation::decode(legacy.encode_to_vec().as_slice())
            .expect("legacy mutation decodes through high-capacity wire type");
        assert_eq!(decoded.content, None);

        let mut counters = DeltaCounters::default();
        let version = validate_atomic_delta(123, atomic(vec![decoded], vec![]), &mut counters)
            .expect("historical empty value remains a value mutation");

        assert_eq!(version.mutations[0].value_hex.as_deref(), Some(""));
        assert_eq!(
            version.mutations[0].value_encoding,
            CertifiedRegistryValueEncoding::Inline
        );
        assert_eq!(counters.inline_value_bytes, 0);
    }

    #[test]
    fn accepts_an_empty_batch_only_at_the_certified_latest_version() {
        let tree = fork(label(CURRENT_VERSION_LABEL, leaf(vec![7])), pruned([9; 32]));
        let batch = validate_delta_tree(&tree, 7, 7).expect("empty current batch");
        assert!(batch.versions.is_empty());
        assert!(!batch.more_available);

        let maximum_tree = fork(
            label(CURRENT_VERSION_LABEL, leaf(encode_leb128(u64::MAX))),
            pruned([8; 32]),
        );
        validate_delta_tree(&maximum_tree, u64::MAX, u64::MAX)
            .expect("empty maximum-version batch");

        let error = validate_delta_tree(&tree, 6, 7).expect_err("missing required delta");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("not proven")
        ));
    }

    #[test]
    fn rejects_wrong_first_gap_duplicate_and_regressing_versions() {
        for (leaves, expected) in [
            (
                vec![delta_leaf(4, atomic(vec![delete(b"a")], vec![]))],
                "expected 3",
            ),
            (
                vec![
                    delta_leaf(3, atomic(vec![delete(b"a")], vec![])),
                    delta_leaf(5, atomic(vec![delete(b"b")], vec![])),
                ],
                "expected 4",
            ),
            (
                vec![
                    delta_leaf(3, atomic(vec![delete(b"a")], vec![])),
                    delta_leaf(3, atomic(vec![delete(b"b")], vec![])),
                ],
                "expected 4",
            ),
            (
                vec![
                    delta_leaf(3, atomic(vec![delete(b"a")], vec![])),
                    delta_leaf(2, atomic(vec![delete(b"b")], vec![])),
                ],
                "expected 4",
            ),
        ] {
            let tree = registry_tree(5, leaves);
            let error = validate_delta_tree(&tree, 2, 5).expect_err("invalid sequence");
            assert!(matches!(
                error,
                RegistryFetchError::InvalidCertifiedRegistry { reason }
                    if reason.contains(expected)
            ));
        }
    }

    #[test]
    fn rejects_unknown_types_and_noncanonical_key_order() {
        let unknown = HighCapacityRegistryMutation {
            mutation_type: 99,
            key: b"alpha".to_vec(),
            content: Some(high_capacity_registry_mutation::Content::Value(vec![1])),
        };
        let error = validate_atomic_delta(
            3,
            atomic(vec![unknown], vec![]),
            &mut DeltaCounters::default(),
        )
        .expect_err("unknown type");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("unknown mutation type 99")
        ));

        let error = validate_atomic_delta(
            5,
            atomic(vec![delete(b"z"), delete(b"a")], vec![]),
            &mut DeltaCounters::default(),
        )
        .expect_err("noncanonical mutation key order");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("out of canonical order")
        ));
    }

    #[test]
    fn accepts_repeated_committed_keys_in_their_stable_order() {
        let version = validate_atomic_delta(
            7_056,
            atomic(
                vec![upsert(b"same", b"first"), upsert(b"same", b"last")],
                vec![],
            ),
            &mut DeltaCounters::default(),
        )
        .expect("same-key committed mutations retain their stable order");

        assert_eq!(version.mutations.len(), 2);
        assert_eq!(version.mutations[0].value_hex, Some(hex_bytes(b"first")));
        assert_eq!(version.mutations[1].value_hex, Some(hex_bytes(b"last")));
    }

    #[test]
    fn accepts_and_counts_certified_chunk_references() {
        let chunk_sha256s = vec![vec![1; 32], vec![2; 32]];
        let chunked = HighCapacityRegistryMutation {
            mutation_type: RegistryMutationType::Upsert as i32,
            key: b"large".to_vec(),
            content: Some(
                high_capacity_registry_mutation::Content::LargeValueChunkKeys(
                    crate::ic_registry::proto::LargeValueChunkKeys {
                        chunk_content_sha256s: chunk_sha256s.clone(),
                    },
                ),
            ),
        };
        let mut counters = DeltaCounters::default();
        let version = validate_atomic_delta(4, atomic(vec![chunked], vec![]), &mut counters)
            .expect("valid chunk references");

        assert_eq!(counters.chunk_reference_count, 2);
        assert_eq!(version.mutations[0].value_hex, None);
        assert_eq!(
            version.mutations[0].value_encoding,
            CertifiedRegistryValueEncoding::Chunked
        );
        assert_eq!(version.mutations[0].chunk_sha256s, chunk_sha256s);
    }

    #[test]
    fn preserves_ignored_delete_content_and_rejects_resource_overflow() {
        let valued_delete = HighCapacityRegistryMutation {
            mutation_type: RegistryMutationType::Delete as i32,
            key: b"alpha".to_vec(),
            content: Some(high_capacity_registry_mutation::Content::Value(vec![1])),
        };
        let version = validate_atomic_delta(
            3,
            atomic(vec![valued_delete], vec![]),
            &mut DeltaCounters::default(),
        )
        .expect("committed delete content is retained but ignored by replay");
        assert_eq!(version.mutations[0].value_hex.as_deref(), Some("01"));
        assert_eq!(
            version.mutations[0].value_encoding,
            CertifiedRegistryValueEncoding::Inline
        );

        let mut counters = DeltaCounters {
            inline_value_bytes: MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES,
            ..DeltaCounters::default()
        };
        let error = validate_atomic_delta(
            3,
            atomic(vec![upsert(b"alpha", b"x")], vec![]),
            &mut counters,
        )
        .expect_err("inline value cap");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("inline value bytes")
        ));
    }

    #[test]
    fn rejects_every_explicit_collection_ceiling() {
        let error = validate_visible_version_count(MAX_CERTIFIED_DELTA_VERSIONS + 1)
            .expect_err("version ceiling");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("visible versions")
        ));

        for (field, maximum) in [
            ("mutation count", MAX_CERTIFIED_DELTA_MUTATIONS),
            ("precondition count", MAX_CERTIFIED_DELTA_PRECONDITIONS),
        ] {
            let mut total = maximum;
            let error =
                checked_accumulate(field, &mut total, 1, maximum).expect_err("count ceiling");
            assert!(matches!(
                error,
                RegistryFetchError::InvalidCertifiedRegistry { reason }
                    if reason.contains(field)
            ));
        }

        let oversized_key = vec![0; MAX_CERTIFIED_DELTA_KEY_BYTES + 1];
        let error = validate_key(1, "mutation", &oversized_key).expect_err("key ceiling");
        assert!(matches!(
            error,
            RegistryFetchError::InvalidCertifiedRegistry { reason }
                if reason.contains("maximum")
        ));
    }

    fn registry_tree(latest: u64, leaves: Vec<HashTree<Vec<u8>>>) -> HashTree<Vec<u8>> {
        fork(
            label(CURRENT_VERSION_LABEL, leaf(encode_leb128(latest))),
            label(DELTA_LABEL, fork_all(leaves)),
        )
    }

    fn delta_leaf(
        version: u64,
        atomic: HighCapacityRegistryAtomicMutateRequest,
    ) -> HashTree<Vec<u8>> {
        label(version.to_be_bytes(), leaf(atomic.encode_to_vec()))
    }

    fn fork_all(mut trees: Vec<HashTree<Vec<u8>>>) -> HashTree<Vec<u8>> {
        let first = trees.drain(..1).next().unwrap_or_else(empty);
        trees.into_iter().fold(first, fork)
    }

    fn atomic(
        mutations: Vec<HighCapacityRegistryMutation>,
        preconditions: Vec<RegistryPrecondition>,
    ) -> HighCapacityRegistryAtomicMutateRequest {
        HighCapacityRegistryAtomicMutateRequest {
            mutations,
            preconditions,
            timestamp_nanoseconds: 123,
        }
    }

    fn upsert(key: &[u8], value: &[u8]) -> HighCapacityRegistryMutation {
        HighCapacityRegistryMutation {
            mutation_type: RegistryMutationType::Upsert as i32,
            key: key.to_vec(),
            content: Some(high_capacity_registry_mutation::Content::Value(
                value.to_vec(),
            )),
        }
    }

    fn delete(key: &[u8]) -> HighCapacityRegistryMutation {
        HighCapacityRegistryMutation {
            mutation_type: RegistryMutationType::Delete as i32,
            key: key.to_vec(),
            content: None,
        }
    }

    fn encode_leb128(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                return bytes;
            }
        }
    }
}
