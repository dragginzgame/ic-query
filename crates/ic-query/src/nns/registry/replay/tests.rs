use super::*;
use crate::{
    nns::registry::{
        NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaSource,
        NnsCertifiedRegistryDeltaSourceFuture, NnsCertifiedRegistryDeltaVersion,
        NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding,
        NnsRegistryCertification, NnsRegistryHostError, nns_certified_registry_delta_limits,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs},
};
use std::sync::Mutex;

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

#[test]
fn replay_session_pins_its_first_target_and_ignores_newer_mutations() {
    let limits =
        NnsRegistryReplaySessionLimits::new(3, 2, 2, 128, NnsRegistryReplayLimits::new(10, 100));
    let mut session = NnsRegistryReplaySession::new(limits);
    let first_request = request(0);
    let first = report_versions(
        &first_request,
        3,
        vec![
            version(
                1,
                vec![mutation(
                    NnsCertifiedRegistryMutationKind::Upsert,
                    b"a",
                    Some(b"one"),
                )],
            ),
            version(
                2,
                vec![mutation(
                    NnsCertifiedRegistryMutationKind::Update,
                    b"a",
                    Some(b"two"),
                )],
            ),
        ],
    );

    let first_progress = session
        .apply_batch(&first_request, &first)
        .expect("partial exact-target replay");

    assert_eq!(session.selected_version(), Some(3));
    assert_eq!(session.state().through_version(), 2);
    assert_eq!(first_progress.applied_version_count, 2);
    assert!(!session.is_complete());

    let second_request = request(2);
    let second = report_versions(
        &second_request,
        4,
        vec![
            version(
                3,
                vec![mutation(
                    NnsCertifiedRegistryMutationKind::Update,
                    b"a",
                    Some(b"three"),
                )],
            ),
            version(
                4,
                vec![mutation(
                    NnsCertifiedRegistryMutationKind::Upsert,
                    b"future",
                    Some(b"ignored"),
                )],
            ),
        ],
    );

    let second_progress = session
        .apply_batch(&second_request, &second)
        .expect("target completion from a newer observation");

    assert_eq!(second_progress.applied_version_count, 1);
    assert_eq!(second_progress.applied_mutation_count, 1);
    assert_eq!(session.state().through_version(), 3);
    assert_eq!(session.state().get(b"a").expect("a").value(), b"three");
    assert!(session.state().get(b"future").is_none());
    assert_eq!(session.highest_certified_latest_version(), Some(4));
    assert_eq!(session.batch_count(), 2);
    assert_eq!(session.query_call_count(), 2);
    assert_eq!(session.response_bytes(), 128);
    assert_eq!(session.applied_mutation_count(), 3);
    assert!(session.is_complete());

    let third_request = request(3);
    let third = report_versions(
        &third_request,
        4,
        vec![version(
            4,
            vec![mutation(
                NnsCertifiedRegistryMutationKind::Upsert,
                b"future",
                Some(b"ignored"),
            )],
        )],
    );
    let complete_error = session
        .apply_batch(&third_request, &third)
        .expect_err("completed session cannot accept another batch");
    assert!(matches!(
        complete_error,
        NnsRegistryReplayError::SessionComplete {
            selected_version: 3
        }
    ));
}

#[test]
fn replay_session_fails_atomically_on_cumulative_limits() {
    let request = request(0);
    let report = report(
        &request,
        3,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"a",
            Some(b"one"),
        )],
        Vec::new(),
    );
    let state_limits = NnsRegistryReplayLimits::new(10, 100);
    let cases = [
        (
            NnsRegistryReplaySessionLimits::new(2, 1, 1, 64, state_limits),
            "selected Registry versions",
            2,
            3,
        ),
        (
            NnsRegistryReplaySessionLimits::new(3, 0, 1, 64, state_limits),
            "batch count",
            0,
            1,
        ),
        (
            NnsRegistryReplaySessionLimits::new(3, 1, 0, 64, state_limits),
            "query call count",
            0,
            1,
        ),
        (
            NnsRegistryReplaySessionLimits::new(3, 1, 1, 63, state_limits),
            "response bytes",
            63,
            64,
        ),
    ];

    for (limits, field, maximum, actual) in cases {
        let mut session = NnsRegistryReplaySession::new(limits);
        let error = session
            .apply_batch(&request, &report)
            .expect_err("cumulative limit must reject the batch");
        assert!(matches!(
            error,
            NnsRegistryReplayError::SessionLimitExceeded {
                field: actual_field,
                maximum: actual_maximum,
                actual: actual_value,
            } if actual_field == field && actual_maximum == maximum && actual_value == actual
        ));
        assert_eq!(session.selected_version(), None);
        assert_eq!(session.batch_count(), 0);
        assert!(session.state().is_empty());
    }
}

#[test]
fn replay_session_rejects_regressing_target_and_changed_root_key() {
    let limits =
        NnsRegistryReplaySessionLimits::new(3, 3, 3, 192, NnsRegistryReplayLimits::new(10, 100));
    let first_request = request(0);
    let first = report(
        &first_request,
        3,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"a",
            Some(b"one"),
        )],
        Vec::new(),
    );

    let second_request = request(1);
    let regressed = report(
        &second_request,
        2,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Update,
            b"a",
            Some(b"two"),
        )],
        Vec::new(),
    );
    let mut session = NnsRegistryReplaySession::new(limits);
    session
        .apply_batch(&first_request, &first)
        .expect("seed session");
    let error = session
        .apply_batch(&second_request, &regressed)
        .expect_err("selected target must remain certified");
    assert!(matches!(
        error,
        NnsRegistryReplayError::CertifiedVersionRegressed {
            selected_version: 3,
            certified_latest_version: 2,
        }
    ));
    assert_eq!(session.state().through_version(), 1);
    assert_eq!(session.batch_count(), 1);

    let mut changed_root = report(
        &second_request,
        3,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Update,
            b"a",
            Some(b"two"),
        )],
        Vec::new(),
    );
    changed_root.certification.root_key_digest = "cd".repeat(32);
    let error = session
        .apply_batch(&second_request, &changed_root)
        .expect_err("root key must remain stable");
    assert!(matches!(
        error,
        NnsRegistryReplayError::RootKeyDigestMismatch { .. }
    ));
    assert_eq!(session.state().through_version(), 1);
    assert_eq!(session.batch_count(), 1);
}

#[test]
fn certified_bootstrap_reserves_each_call_and_completes_the_first_exact_target() {
    let source = BootstrapSource::default();
    let request = bootstrap_request(MAINNET_NETWORK, 2, 130, 80 * 1_024 * 1_024);

    let session = futures::executor::block_on(bootstrap_nns_certified_registry_with_source_async(
        &request, &source,
    ))
    .expect("bounded fixture bootstrap");

    assert_eq!(source.requested_versions(), vec![0, 2]);
    assert_eq!(session.selected_version(), Some(3));
    assert_eq!(session.highest_certified_latest_version(), Some(4));
    assert_eq!(session.state().through_version(), 3);
    assert_eq!(session.state().get(b"a").expect("a").value(), b"three");
    assert!(session.state().get(b"future").is_none());
    assert_eq!(session.batch_count(), 2);
    assert_eq!(session.query_call_count(), 2);
    assert_eq!(session.response_bytes(), 128);
    assert!(session.is_complete());

    let probe_source = BootstrapSource::default();
    let outcome = futures::executor::block_on(probe_nns_certified_registry_with_source_async(
        &request,
        &probe_source,
    ))
    .expect("complete diagnostic probe");
    assert_eq!(
        outcome.status,
        NnsCertifiedRegistryBootstrapProbeStatus::Complete
    );
    assert!(outcome.session.is_complete());
    assert_eq!(probe_source.requested_versions(), vec![0, 2]);
}

#[test]
fn certified_bootstrap_probe_returns_explicit_bounded_partial_progress() {
    let source = BootstrapSource::default();
    let request = bootstrap_request(MAINNET_NETWORK, 1, 65, 40 * 1_024 * 1_024);

    let outcome = futures::executor::block_on(probe_nns_certified_registry_with_source_async(
        &request, &source,
    ))
    .expect("bounded incomplete diagnostic probe");

    assert_eq!(
        outcome.status,
        NnsCertifiedRegistryBootstrapProbeStatus::CapacityReached {
            field: "batch count",
            maximum: 1,
            required: 2,
        }
    );
    assert_eq!(source.requested_versions(), vec![0]);
    assert_eq!(outcome.session.selected_version(), Some(3));
    assert_eq!(outcome.session.state().through_version(), 2);
    assert_eq!(outcome.session.batch_count(), 1);
    assert_eq!(outcome.session.query_call_count(), 1);
    assert_eq!(outcome.session.response_bytes(), 64);
    assert!(!outcome.session.is_complete());

    let zero_source = BootstrapSource::default();
    let zero_request = bootstrap_request(MAINNET_NETWORK, 0, 0, 0);
    let zero = futures::executor::block_on(probe_nns_certified_registry_with_source_async(
        &zero_request,
        &zero_source,
    ))
    .expect("zero-call diagnostic probe");
    assert_eq!(
        zero.status,
        NnsCertifiedRegistryBootstrapProbeStatus::CapacityReached {
            field: "batch count",
            maximum: 0,
            required: 1,
        }
    );
    assert_eq!(zero.session.selected_version(), None);
    assert!(zero_source.requested_versions().is_empty());
}

#[test]
fn certified_bootstrap_never_starts_a_batch_without_worst_case_capacity() {
    let mebibyte = 1_024 * 1_024;
    let cases = [
        (1, 130, 80 * mebibyte, "batch count", 1, 2),
        (2, 65, 80 * mebibyte, "query call count", 65, 66),
        (
            2,
            130,
            40 * mebibyte,
            "response bytes",
            40 * mebibyte,
            40 * mebibyte + 64,
        ),
    ];

    for (max_batches, max_calls, max_bytes, field, maximum, actual) in cases {
        let source = BootstrapSource::default();
        let request = bootstrap_request(MAINNET_NETWORK, max_batches, max_calls, max_bytes);
        let error = futures::executor::block_on(
            bootstrap_nns_certified_registry_with_source_async(&request, &source),
        )
        .expect_err("second batch lacks worst-case reservation");

        assert!(matches!(
            error,
            NnsRegistryReplayError::SessionLimitExceeded {
                field: actual_field,
                maximum: actual_maximum,
                actual: actual_value,
            } if actual_field == field && actual_maximum == maximum && actual_value == actual
        ));
        assert_eq!(source.requested_versions(), vec![0]);
    }
}

#[test]
fn certified_bootstrap_rejects_non_mainnet_before_source_work() {
    let source = BootstrapSource::default();
    let request = bootstrap_request("local", 0, 0, 0);

    let error = futures::executor::block_on(bootstrap_nns_certified_registry_with_source_async(
        &request, &source,
    ))
    .expect_err("non-mainnet bootstrap");

    assert!(matches!(
        error,
        NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork {
            network
        }) if network == "local"
    ));
    assert!(source.requested_versions().is_empty());

    let live_error = futures::executor::block_on(bootstrap_nns_certified_registry_async(&request))
        .expect_err("live non-mainnet bootstrap");
    assert!(matches!(
        live_error,
        NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork {
            network
        }) if network == "local"
    ));

    let probe_error = futures::executor::block_on(probe_nns_certified_registry_async(&request))
        .expect_err("live non-mainnet probe");
    assert!(matches!(
        probe_error,
        NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork {
            network
        }) if network == "local"
    ));
}

#[derive(Default)]
struct BootstrapSource {
    requested_versions: Mutex<Vec<u64>>,
}

impl BootstrapSource {
    fn requested_versions(&self) -> Vec<u64> {
        self.requested_versions
            .lock()
            .expect("bootstrap fixture request lock")
            .clone()
    }
}

impl NnsCertifiedRegistryDeltaSource for BootstrapSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        self.requested_versions
            .lock()
            .expect("bootstrap fixture request lock")
            .push(request.requested_version);
        Box::pin(async move {
            match request.requested_version {
                0 => Ok(report_versions(
                    request,
                    3,
                    vec![
                        version(
                            1,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"a",
                                Some(b"one"),
                            )],
                        ),
                        version(
                            2,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Update,
                                b"a",
                                Some(b"two"),
                            )],
                        ),
                    ],
                )),
                2 => Ok(report_versions(
                    request,
                    4,
                    vec![
                        version(
                            3,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Update,
                                b"a",
                                Some(b"three"),
                            )],
                        ),
                        version(
                            4,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"future",
                                Some(b"ignored"),
                            )],
                        ),
                    ],
                )),
                version => Err(NnsRegistryHostError::InvalidSourceData {
                    reason: format!("unexpected bootstrap fixture request after version {version}"),
                }),
            }
        })
    }
}

fn bootstrap_request(
    network: &str,
    max_batches: u64,
    max_query_calls: u64,
    max_response_bytes: u64,
) -> NnsCertifiedRegistryBootstrapRequest {
    NnsCertifiedRegistryBootstrapRequest::new(
        network,
        "https://icp-api.io",
        NOW,
        NnsRegistryReplaySessionLimits::new(
            3,
            max_batches,
            max_query_calls,
            max_response_bytes,
            NnsRegistryReplayLimits::new(10, 100),
        ),
    )
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
    report_versions(
        request,
        certified_latest_version,
        vec![NnsCertifiedRegistryDeltaVersion {
            version,
            timestamp_nanoseconds: NOW * 1_000_000_000,
            mutations,
            preconditions,
        }],
    )
}

fn report_versions(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    certified_latest_version: u64,
    versions: Vec<NnsCertifiedRegistryDeltaVersion>,
) -> NnsCertifiedRegistryDeltaBatchReport {
    let inline_value_bytes = versions
        .iter()
        .flat_map(|version| &version.mutations)
        .filter_map(|mutation| mutation.value_hex.as_ref())
        .map(|value| value.len() / 2)
        .sum();
    let mutation_count = versions.iter().map(|version| version.mutations.len()).sum();
    let precondition_count = versions
        .iter()
        .map(|version| version.preconditions.len())
        .sum();
    let first_version = versions.first().map(|version| version.version);
    let last_version = versions.last().map(|version| version.version);
    NnsCertifiedRegistryDeltaBatchReport {
        schema_version: 2,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: request.requested_version,
        certified_latest_version,
        first_version,
        last_version,
        version_count: versions.len(),
        mutation_count,
        precondition_count,
        inline_value_bytes,
        chunk_value_bytes: 0,
        value_bytes: inline_value_bytes,
        chunk_reference_count: 0,
        more_available: last_version.unwrap_or(request.requested_version)
            < certified_latest_version,
        fetched_at: format_utc_timestamp_secs(NOW),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,
        response_bytes: 64,
        limits: limits(),
        versions,
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

fn version(
    version: u64,
    mutations: Vec<NnsCertifiedRegistryMutation>,
) -> NnsCertifiedRegistryDeltaVersion {
    NnsCertifiedRegistryDeltaVersion {
        version,
        timestamp_nanoseconds: NOW * 1_000_000_000,
        mutations,
        preconditions: Vec::new(),
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
