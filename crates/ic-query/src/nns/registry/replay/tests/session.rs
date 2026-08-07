//! Module: nns::registry::replay::tests::session
//!
//! Responsibility: Registry replay state, provenance, and bounded-session tests.
//! Does not own: production replay behavior or shared protocol fixtures.
//! Boundary: exercises the corresponding replay subsystem through fixture evidence.

use super::*;

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
fn replay_applies_repeated_committed_keys_in_stable_order() {
    let limits = NnsRegistryReplayLimits::new(10, 100);
    let request = request(0);
    let report = report(
        &request,
        1,
        1,
        vec![
            mutation(
                NnsCertifiedRegistryMutationKind::Upsert,
                b"same",
                Some(b"x"),
            ),
            mutation(
                NnsCertifiedRegistryMutationKind::Upsert,
                b"same",
                Some(b"last"),
            ),
        ],
        Vec::new(),
    );
    let mut state = NnsRegistryReplayState::new();

    let progress = apply_nns_certified_registry_delta_batch(&mut state, &request, &report, limits)
        .expect("repeated committed key replay");

    assert_eq!(progress.applied_mutation_count, 2);
    let value = state.get(b"same").expect("final same-key value");
    assert_eq!(value.value(), b"last");
    assert_eq!(value.last_mutation_version(), 1);
    assert_eq!(state.entry_count(), 1);
    assert_eq!(state.content_bytes(), 8);

    let mut rejected = NnsRegistryReplayState::new();
    let error = apply_nns_certified_registry_delta_batch(
        &mut rejected,
        &request,
        &report,
        NnsRegistryReplayLimits::new(10, 7),
    )
    .expect_err("final same-key value exceeds content limit");
    assert!(matches!(
        error,
        NnsRegistryReplayError::LimitExceeded {
            field: "content bytes",
            maximum: 7,
            actual: 8,
        }
    ));
    assert_eq!(rejected.through_version(), 0);
    assert!(rejected.is_empty());
}

#[test]
fn replay_ignores_retained_content_on_a_committed_delete() {
    let limits = NnsRegistryReplayLimits::new(10, 100);
    let first_request = request(0);
    let first = report(
        &first_request,
        2,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"key",
            Some(b"present"),
        )],
        Vec::new(),
    );
    let second_request = request(1);
    let second = report(
        &second_request,
        2,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Delete,
            b"key",
            Some(b"historical ignored bytes"),
        )],
        Vec::new(),
    );
    let mut state = NnsRegistryReplayState::new();

    apply_nns_certified_registry_delta_batch(&mut state, &first_request, &first, limits)
        .expect("initial value");
    let progress =
        apply_nns_certified_registry_delta_batch(&mut state, &second_request, &second, limits)
            .expect("committed delete with retained content");

    assert_eq!(progress.applied_mutation_count, 1);
    assert_eq!(state.through_version(), 2);
    assert!(state.get(b"key").is_none());
    assert!(state.is_empty());
    assert_eq!(state.content_bytes(), 0);
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
    assert_eq!(session.complete_state_digest(), None);

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
    assert!(session.complete_state_digest().is_some());

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
fn replay_session_publishes_provenance_atomically() {
    let fixture = provenance_fixture();
    let mut session = NnsRegistryReplaySession::new(fixture.limits);

    assert_eq!(session.evidence_chain_digest(), None);
    assert_eq!(session.complete_state_digest(), None);
    assert_eq!(session.minimum_certificate_time_nanos(), None);
    assert_eq!(session.maximum_certificate_time_nanos(), None);
    assert_eq!(session.source_endpoints().count(), 0);

    session
        .apply_batch(&fixture.first_request, &fixture.first)
        .expect("first provenance batch");
    let first_evidence_digest = session
        .evidence_chain_digest()
        .expect("partial evidence digest");
    assert_eq!(session.complete_state_digest(), None);
    assert_eq!(
        session.minimum_certificate_time_nanos(),
        Some(NOW * 1_000_000_000)
    );
    assert_eq!(
        session.maximum_certificate_time_nanos(),
        Some(NOW * 1_000_000_000)
    );
    assert_eq!(
        session.source_endpoints().collect::<Vec<_>>(),
        vec!["https://icp-api.io"]
    );

    let oversized_value = vec![0; 100];
    let failed = report(
        &fixture.second_request,
        2,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"b",
            Some(&oversized_value),
        )],
        Vec::new(),
    );
    let state_before_failure = session.state().clone();
    let error = session
        .apply_batch(&fixture.second_request, &failed)
        .expect_err("failed replay does not publish provenance");
    assert!(matches!(
        error,
        NnsRegistryReplayError::LimitExceeded {
            field: "content bytes",
            maximum: 100,
            actual: 105,
        }
    ));
    assert_eq!(session.state(), &state_before_failure);
    assert_eq!(session.evidence_chain_digest(), Some(first_evidence_digest));
    assert_eq!(session.complete_state_digest(), None);
    assert_eq!(
        session.source_endpoints().collect::<Vec<_>>(),
        vec!["https://icp-api.io"]
    );
    assert_eq!(
        session.minimum_certificate_time_nanos(),
        Some(NOW * 1_000_000_000)
    );
    assert_eq!(
        session.maximum_certificate_time_nanos(),
        Some(NOW * 1_000_000_000)
    );
    assert_eq!(session.batch_count(), 1);

    session
        .apply_batch(&fixture.second_request, &fixture.second)
        .expect("complete provenance batch");
    let evidence_digest = session
        .evidence_chain_digest()
        .expect("complete evidence digest");
    assert!(session.complete_state_digest().is_some());
    assert_ne!(evidence_digest, first_evidence_digest);
    assert_eq!(
        session.minimum_certificate_time_nanos(),
        Some((NOW - 60) * 1_000_000_000)
    );
    assert_eq!(
        session.maximum_certificate_time_nanos(),
        Some(NOW * 1_000_000_000)
    );
    assert_eq!(
        session.source_endpoints().collect::<Vec<_>>(),
        vec!["https://example.com", "https://icp-api.io"]
    );
    assert!(session.is_complete());
}

#[test]
fn replay_session_provenance_digests_are_deterministic_and_domain_stable() {
    let baseline = complete_provenance_session(false);
    let evidence_digest = baseline
        .evidence_chain_digest()
        .expect("complete evidence digest");
    let state_digest = baseline
        .complete_state_digest()
        .expect("complete state digest");

    let same = complete_provenance_session(false);
    assert_eq!(same.evidence_chain_digest(), Some(evidence_digest));
    assert_eq!(same.complete_state_digest(), Some(state_digest));

    let changed = complete_provenance_session(true);
    assert_ne!(changed.evidence_chain_digest(), Some(evidence_digest));
    assert_eq!(changed.complete_state_digest(), Some(state_digest));

    assert_eq!(
        crate::hex::hex_bytes(&evidence_digest),
        "d2c7f253eaef14275bffacf96173d04dfe98fdd818c0c1fcdd416ae419479531"
    );
    assert_eq!(
        crate::hex::hex_bytes(&state_digest),
        "c7cb524c6317bb40d117ea0a3375345e44412afd3d03dc0eee810be5c7c0a705"
    );
}

#[test]
fn reauthenticated_batches_build_one_complete_authenticated_session() {
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let mut builder = NnsAuthenticatedRegistryReplayBuilder::new(fixture.limits);

    let first_progress = builder
        .apply_batch(&first)
        .expect("first reauthenticated retained batch");
    assert_eq!(first_progress.through_version, 1);
    assert!(!builder.replay_session().is_complete());

    let second_progress = builder
        .apply_batch(&second)
        .expect("second reauthenticated retained batch");
    assert_eq!(second_progress.through_version, 2);
    assert!(builder.replay_session().is_complete());

    let authenticated = builder
        .into_authenticated_replay_session()
        .expect("complete retained replay can be sealed");
    let session = authenticated.replay_session();
    assert_eq!(session.selected_version(), Some(2));
    assert_eq!(session.batch_count(), 2);
    assert_eq!(session.source_endpoints().count(), 2);
    assert!(session.evidence_chain_digest().is_some());
    assert!(session.complete_state_digest().is_some());
}

#[test]
fn reauthenticated_replay_builder_rejects_incomplete_and_over_limit_sequences() {
    let fixture = provenance_fixture();
    let empty = NnsAuthenticatedRegistryReplayBuilder::new(fixture.limits);
    let error = empty
        .into_authenticated_replay_session()
        .expect_err("empty retained replay cannot be sealed");
    assert!(matches!(
        error,
        NnsRegistryReplayError::AuthenticationRequiresCompleteSession {
            selected_version: None,
            through_version: 0,
        }
    ));

    let limited =
        NnsRegistryReplaySessionLimits::new(2, 1, 2, 128, NnsRegistryReplayLimits::new(10, 100));
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let mut builder = NnsAuthenticatedRegistryReplayBuilder::new(limited);
    builder
        .apply_batch(&first)
        .expect("first retained batch fits cumulative limits");
    let error = builder
        .apply_batch(&second)
        .expect_err("second retained batch exceeds cumulative limit");
    assert!(matches!(
        error,
        NnsRegistryReplayError::SessionLimitExceeded {
            field: "batch count",
            maximum: 1,
            actual: 2,
        }
    ));
    assert_eq!(builder.replay_session().state().through_version(), 1);
    assert_eq!(builder.replay_session().batch_count(), 1);
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
        assert_eq!(session.evidence_chain_digest(), None);
        assert_eq!(session.complete_state_digest(), None);
        assert_eq!(session.source_endpoints().count(), 0);
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
