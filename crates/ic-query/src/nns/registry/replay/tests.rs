use super::*;
use crate::{
    nns::registry::{
        NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaVersion,
        NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding,
        NnsRegistryCertification, nns_certified_registry_delta_limits,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};

const NOW: u64 = 1_780_531_200;

#[test]
fn replay_applies_committed_batches_in_order_and_tracks_current_state() {
    let limits = NnsRegistryReplayLimits::new(10, 100);
    let mut state = NnsRegistryReplayState::new();
    let first_request = request(0);
    let first = report(
        &first_request,
        2,
        1,
        vec![
            mutation(NnsCertifiedRegistryMutationKind::Upsert, b"a", Some(b"one")),
            mutation(NnsCertifiedRegistryMutationKind::Upsert, b"b", Some(b"two")),
        ],
        Vec::new(),
    );

    let progress =
        apply_nns_certified_registry_delta_batch(&mut state, &first_request, &first, limits)
            .expect("first committed batch");

    assert_eq!(progress.previous_version, 0);
    assert_eq!(progress.through_version, 1);
    assert!(!progress.complete_at_certified_latest_version);
    assert_eq!(state.content_bytes(), 8);

    let second_request = request(1);
    let second = report(
        &second_request,
        2,
        2,
        vec![
            mutation(
                NnsCertifiedRegistryMutationKind::Insert,
                b"a",
                Some(b"three"),
            ),
            mutation(NnsCertifiedRegistryMutationKind::Delete, b"b", None),
            mutation(NnsCertifiedRegistryMutationKind::Update, b"c", Some(b"")),
        ],
        vec![NnsCertifiedRegistryPrecondition {
            key_hex: crate::hex::hex_bytes(b"a"),
            expected_version: 999,
        }],
    );

    let progress =
        apply_nns_certified_registry_delta_batch(&mut state, &second_request, &second, limits)
            .expect("second committed batch");

    assert!(progress.complete_at_certified_latest_version);
    assert_eq!(progress.applied_mutation_count, 3);
    assert_eq!(state.through_version(), 2);
    assert_eq!(state.entry_count(), 2);
    assert_eq!(state.content_bytes(), 7);
    assert_eq!(state.get(b"a").expect("a").value(), b"three");
    assert_eq!(state.get(b"a").expect("a").last_mutation_version(), 2);
    assert!(state.get(b"b").is_none());
    assert_eq!(state.get(b"c").expect("c").value(), b"");
    assert_eq!(
        state
            .entries()
            .map(|(key, _)| key.to_vec())
            .collect::<Vec<_>>(),
        vec![b"a".to_vec(), b"c".to_vec()]
    );
}

#[test]
fn replay_rejects_version_mismatch_and_rolls_back_limit_failure() {
    let generous = NnsRegistryReplayLimits::new(10, 100);
    let first_request = request(0);
    let first = report(
        &first_request,
        2,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"a",
            Some(b"one"),
        )],
        Vec::new(),
    );
    let mut state = NnsRegistryReplayState::new();
    apply_nns_certified_registry_delta_batch(&mut state, &first_request, &first, generous)
        .expect("seed state");

    let before = state.clone();
    let second_request = request(1);
    let second = report(
        &second_request,
        2,
        2,
        vec![
            mutation(NnsCertifiedRegistryMutationKind::Update, b"a", Some(b"x")),
            mutation(
                NnsCertifiedRegistryMutationKind::Upsert,
                b"bb",
                Some(b"four"),
            ),
        ],
        Vec::new(),
    );
    let error = apply_nns_certified_registry_delta_batch(
        &mut state,
        &second_request,
        &second,
        NnsRegistryReplayLimits::new(10, 7),
    )
    .expect_err("content ceiling");
    assert!(matches!(
        error,
        NnsRegistryReplayError::LimitExceeded {
            field: "content bytes",
            maximum: 7,
            actual: 8,
        }
    ));
    assert_eq!(state, before);

    let error = apply_nns_certified_registry_delta_batch(
        &mut state,
        &second_request,
        &second,
        NnsRegistryReplayLimits::new(1, 100),
    )
    .expect_err("entry ceiling");
    assert!(matches!(
        error,
        NnsRegistryReplayError::LimitExceeded {
            field: "entry count",
            maximum: 1,
            actual: 2,
        }
    ));
    assert_eq!(state, before);

    let mut empty = NnsRegistryReplayState::new();
    let error =
        apply_nns_certified_registry_delta_batch(&mut empty, &second_request, &second, generous)
            .expect_err("version mismatch");
    assert!(matches!(
        error,
        NnsRegistryReplayError::VersionMismatch {
            state_version: 0,
            requested_version: 1,
        }
    ));
    assert!(empty.is_empty());
}

fn request(requested_version: u64) -> NnsCertifiedRegistryDeltaBatchRequest {
    NnsCertifiedRegistryDeltaBatchRequest::new(
        MAINNET_NETWORK,
        "https://icp-api.io",
        requested_version,
        NOW,
    )
}

fn report(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    certified_latest_version: u64,
    version: u64,
    mutations: Vec<NnsCertifiedRegistryMutation>,
    preconditions: Vec<NnsCertifiedRegistryPrecondition>,
) -> NnsCertifiedRegistryDeltaBatchReport {
    let inline_value_bytes = mutations
        .iter()
        .filter_map(|mutation| mutation.value_hex.as_ref())
        .map(|value| value.len() / 2)
        .sum();
    NnsCertifiedRegistryDeltaBatchReport {
        schema_version: 2,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: request.requested_version,
        certified_latest_version,
        first_version: Some(version),
        last_version: Some(version),
        version_count: 1,
        mutation_count: mutations.len(),
        precondition_count: preconditions.len(),
        inline_value_bytes,
        chunk_value_bytes: 0,
        value_bytes: inline_value_bytes,
        chunk_reference_count: 0,
        more_available: version < certified_latest_version,
        fetched_at: format_utc_timestamp_secs(NOW),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,
        response_bytes: 64,
        limits: limits(),
        versions: vec![NnsCertifiedRegistryDeltaVersion {
            version,
            timestamp_nanoseconds: NOW * 1_000_000_000,
            mutations,
            preconditions,
        }],
        certification: NnsRegistryCertification {
            certificate_verified: true,
            certificate_time_nanos: NOW * 1_000_000_000,
            certificate_time: format_utc_timestamp_secs(NOW),
            root_key_digest: "ab".repeat(32),
            certificate_hex: "cd".repeat(8),
            certificate_bytes: 8,
            hash_tree_hex: "ef".repeat(4),
            hash_tree_bytes: 4,
        },
    }
}

fn mutation(
    kind: NnsCertifiedRegistryMutationKind,
    key: &[u8],
    value: Option<&[u8]>,
) -> NnsCertifiedRegistryMutation {
    NnsCertifiedRegistryMutation {
        mutation_type: match kind {
            NnsCertifiedRegistryMutationKind::Insert => 0,
            NnsCertifiedRegistryMutationKind::Update => 1,
            NnsCertifiedRegistryMutationKind::Delete => 2,
            NnsCertifiedRegistryMutationKind::Upsert => 4,
        },
        mutation_kind: kind,
        key_hex: crate::hex::hex_bytes(key),
        value_encoding: if value.is_some() {
            NnsCertifiedRegistryValueEncoding::Inline
        } else {
            NnsCertifiedRegistryValueEncoding::Absent
        },
        chunk_sha256_hexes: Vec::new(),
        value_hex: value.map(crate::hex::hex_bytes),
    }
}

const fn limits() -> NnsCertifiedRegistryDeltaLimits {
    nns_certified_registry_delta_limits()
}
