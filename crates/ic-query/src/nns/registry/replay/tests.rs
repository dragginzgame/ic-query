use super::*;
use crate::{
    ic_registry::proto::{
        CanisterId, CanisterIdRange, PrincipalId, RoutingTable, RoutingTableEntry, SubnetId,
        SubnetListRecord, SubnetRecord, SubnetType,
    },
    nns::registry::{
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsAuthenticatedRegistryDeltaBatch,
        NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaSource,
        NnsCertifiedRegistryDeltaSourceFuture, NnsCertifiedRegistryDeltaVersion,
        NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding,
        NnsRegistryCertification, NnsRegistryHostError, nns_certified_registry_delta_limits,
    },
    subnet_catalog::{
        CatalogError, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetKind,
        SubnetSpecialization, format_utc_timestamp_secs,
    },
};
use candid::Principal;
use prost::Message;
use std::{fs, sync::Mutex};

const NOW: u64 = 1_780_531_200;
const PROJECTION_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const PROJECTION_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

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
fn complete_replay_projects_through_shared_catalog_classification() {
    let session = complete_catalog_projection_session(true, false);

    let projection =
        project_nns_registry_subnet_catalog(&session).expect("complete catalog projection");

    assert_eq!(projection.registry_version(), 1);
    assert_eq!(projection.replay_session(), &session);
    assert_eq!(projection.subnets().len(), 1);
    let subnet = &projection.subnets()[0];
    assert_eq!(subnet.subnet_principal, PROJECTION_SUBNET);
    assert_eq!(subnet.subnet_kind, SubnetKind::Application);
    assert_eq!(
        subnet.subnet_specialization,
        SubnetSpecialization::Fiduciary
    );
    assert_eq!(subnet.node_count, Some(2));
    assert_eq!(projection.routing_ranges().len(), 1);
    assert_eq!(
        projection.routing_ranges()[0].subnet_principal,
        PROJECTION_SUBNET
    );
    assert!(
        projection
            .replay_session()
            .evidence_chain_digest()
            .is_some()
    );
    assert!(
        projection
            .replay_session()
            .complete_state_digest()
            .is_some()
    );
}

#[test]
fn catalog_projection_rejects_incomplete_and_missing_replay_state() {
    let empty = NnsRegistryReplaySession::new(NnsRegistryReplaySessionLimits::new(
        1,
        1,
        1,
        64,
        NnsRegistryReplayLimits::new(10, 1_000),
    ));
    let error = project_nns_registry_subnet_catalog(&empty)
        .expect_err("empty replay session is not a catalog snapshot");
    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::IncompleteSession {
            selected_version: None,
            through_version: 0,
        }
    ));

    let request = request(0);
    let report = report(
        &request,
        1,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"unrelated",
            Some(b"value"),
        )],
        Vec::new(),
    );
    let mut complete = projection_session();
    complete
        .apply_batch(&request, &report)
        .expect("complete replay without catalog records");
    let error = project_nns_registry_subnet_catalog(&complete)
        .expect_err("required catalog key remains mandatory");
    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::MissingRequiredRegistryKey { key }
            if key == "subnet_list"
    ));
}

#[test]
fn catalog_projection_rejects_registry_version_zero() {
    let request = request(0);
    let report = report_versions(&request, 0, Vec::new());
    let mut session = NnsRegistryReplaySession::new(NnsRegistryReplaySessionLimits::new(
        0,
        1,
        1,
        64,
        NnsRegistryReplayLimits::new(10, 100),
    ));
    session
        .apply_batch(&request, &report)
        .expect("complete version-zero replay fixture");

    let error = project_nns_registry_subnet_catalog(&session)
        .expect_err("version zero is not a catalog authority position");

    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::InvalidRegistryVersion
    ));
}

#[test]
fn catalog_projection_preserves_typed_record_and_catalog_failures() {
    let invalid = complete_catalog_projection_session(true, true);
    let error =
        project_nns_registry_subnet_catalog(&invalid).expect_err("malformed replayed subnet list");
    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::InvalidRegistryRecord {
            key,
            message: "SubnetListRecord",
            ..
        } if key == "subnet_list"
    ));

    let missing_record = complete_catalog_projection_session(false, false);
    let error = project_nns_registry_subnet_catalog(&missing_record)
        .expect_err("referenced subnet record is required");
    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::MissingRequiredRegistryKey { key }
            if key == format!("subnet_record_{PROJECTION_SUBNET}")
    ));

    let empty_routing = complete_catalog_projection_session_with_routing(RoutingTable::default());
    let error = project_nns_registry_subnet_catalog(&empty_routing)
        .expect_err("empty routing projection fails shared catalog validation");
    assert!(matches!(
        error,
        NnsRegistrySubnetCatalogProjectionError::Catalog(CatalogError::EmptyRoutingRanges)
    ));
}

#[test]
fn authenticated_replay_requires_complete_provenance() {
    let session = projection_session();

    let error = NnsAuthenticatedRegistryReplaySession::from_verified_complete(session)
        .expect_err("incomplete replay cannot acquire authentication capability");

    assert!(matches!(
        error,
        NnsRegistryReplayError::AuthenticationRequiresCompleteSession {
            selected_version: None,
            through_version: 0,
        }
    ));
}

#[test]
fn authenticated_projection_composes_without_copying_catalog_rows() {
    let session = complete_catalog_projection_session(true, false);
    let authenticated = NnsAuthenticatedRegistryReplaySession::from_verified_complete(session)
        .expect("complete verified fixture authentication");

    {
        let projected = project_nns_authenticated_registry_subnet_catalog(&authenticated)
            .expect("authenticated catalog projection");
        assert_eq!(projected.authenticated_replay_session(), &authenticated);
        assert_eq!(projected.projection().registry_version(), 1);
        assert_eq!(projected.projection().subnets().len(), 1);
        assert_eq!(
            projected.projection().subnets()[0].subnet_principal,
            PROJECTION_SUBNET
        );
    }

    let ordinary = authenticated.into_replay_session();
    assert!(ordinary.is_complete());
    assert_eq!(ordinary.selected_version(), Some(1));
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
fn authenticated_archive_manifest_is_canonical_and_bound_to_reports() {
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let archive_limits = NnsCertifiedRegistryArchiveLimits::new(2, 100_000, 200_000);
    let mut builder =
        NnsCertifiedRegistryArchiveManifestBuilder::new(fixture.limits, archive_limits);

    builder.apply_batch(&first).expect("first archive batch");
    builder.apply_batch(&second).expect("second archive batch");
    let (manifest, authenticated) = builder.finish().expect("complete archive manifest");

    assert_eq!(
        manifest.schema_version,
        NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(manifest.network, MAINNET_NETWORK);
    assert_eq!(
        manifest.delta_report_schema_version,
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION
    );
    assert_eq!(
        manifest.replay_provenance_schema_version,
        NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION
    );
    assert_eq!(manifest.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(manifest.selected_version, 2);
    assert_eq!(manifest.batch_count, 2);
    assert_eq!(manifest.batches.len(), 2);
    assert_eq!(manifest.batches[0].ordinal, 0);
    assert_eq!(manifest.batches[0].requested_version, 0);
    assert_eq!(manifest.batches[0].applied_through_version, 1);
    assert_eq!(manifest.batches[1].ordinal, 1);
    assert_eq!(manifest.batches[1].requested_version, 1);
    assert_eq!(manifest.batches[1].applied_through_version, 2);
    assert_eq!(
        manifest.total_report_bytes,
        manifest
            .batches
            .iter()
            .map(|batch| batch.report_bytes)
            .sum::<u64>()
    );
    assert_eq!(
        manifest.source_endpoints,
        ["https://example.com", "https://icp-api.io"]
    );
    assert_eq!(manifest.root_key_digest, "ab".repeat(32));
    assert_eq!(manifest.evidence_chain_digest.len(), 64);
    assert_eq!(manifest.complete_state_digest.len(), 64);
    assert_ne!(
        manifest.batches[0].report_sha256,
        manifest.batches[1].report_sha256
    );
    assert_eq!(
        manifest.batches[0].report_bytes,
        u64::try_from(
            serde_json::to_vec(&fixture.first)
                .expect("canonical first JSON")
                .len()
        )
        .expect("first JSON length")
    );
    validate_nns_certified_registry_archive_manifest(&manifest, archive_limits)
        .expect("built manifest validates");
    let round_trip: NnsCertifiedRegistryArchiveManifest =
        serde_json::from_slice(&serde_json::to_vec(&manifest).expect("serialize archive manifest"))
            .expect("deserialize archive manifest");
    assert_eq!(round_trip, manifest);
    let mut unknown_field = serde_json::to_value(&manifest).expect("manifest JSON value");
    unknown_field
        .as_object_mut()
        .expect("manifest JSON object")
        .insert("future_field".to_string(), serde_json::json!(true));
    assert!(
        serde_json::from_value::<NnsCertifiedRegistryArchiveManifest>(unknown_field).is_err(),
        "schema-1 manifests reject undeclared fields"
    );
    assert_eq!(
        authenticated.replay_session().complete_state_digest(),
        Some(
            crate::hex::decode_lowercase_hex(&manifest.complete_state_digest)
                .expect("state digest hex")
                .try_into()
                .expect("32-byte state digest")
        )
    );
}

#[test]
fn archive_builder_enforces_encoding_limits_before_replay_publication() {
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let first_bytes = u64::try_from(
        serde_json::to_vec(&fixture.first)
            .expect("first report JSON")
            .len(),
    )
    .expect("first report length");
    let limits = NnsCertifiedRegistryArchiveLimits::new(2, first_bytes - 1, first_bytes * 2);
    let mut builder = NnsCertifiedRegistryArchiveManifestBuilder::new(fixture.limits, limits);

    let error = builder
        .apply_batch(&first)
        .expect_err("oversized canonical report");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveError::LimitExceeded {
            field: "batch report bytes",
            maximum,
            actual,
        } if maximum == first_bytes - 1 && actual == first_bytes
    ));
    assert_eq!(builder.replay_session().batch_count(), 0);
    assert_eq!(builder.replay_session().state().through_version(), 0);

    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let second_bytes = u64::try_from(
        serde_json::to_vec(&fixture.second)
            .expect("second report JSON")
            .len(),
    )
    .expect("second report length");
    let total_limit = first_bytes + second_bytes - 1;
    let limits = NnsCertifiedRegistryArchiveLimits::new(2, 100_000, total_limit);
    let mut builder = NnsCertifiedRegistryArchiveManifestBuilder::new(fixture.limits, limits);
    builder
        .apply_batch(&first)
        .expect("first report fits total archive limit");

    let error = builder
        .apply_batch(&second)
        .expect_err("second report exceeds total archive limit");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveError::LimitExceeded {
            field: "total report bytes",
            maximum,
            actual,
        } if maximum == total_limit && actual == first_bytes + second_bytes
    ));
    assert_eq!(builder.replay_session().batch_count(), 1);
    assert_eq!(builder.replay_session().state().through_version(), 1);
}

#[test]
fn archive_manifest_validation_rejects_tampered_index_fields() {
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let limits = NnsCertifiedRegistryArchiveLimits::new(2, 100_000, 200_000);
    let mut builder = NnsCertifiedRegistryArchiveManifestBuilder::new(fixture.limits, limits);
    builder.apply_batch(&first).expect("first archive batch");
    builder.apply_batch(&second).expect("second archive batch");
    let (manifest, _) = builder.finish().expect("complete archive manifest");

    let mut wrong_ordinal = manifest.clone();
    wrong_ordinal.batches[1].ordinal = 0;
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&wrong_ordinal, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut skipped_version = manifest.clone();
    skipped_version.batches[1].requested_version = 0;
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&skipped_version, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut changed_digest = manifest.clone();
    changed_digest.batches[0].report_sha256 = "AB".repeat(32);
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&changed_digest, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut changed_total = manifest.clone();
    changed_total.total_report_bytes += 1;
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&changed_total, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut unsorted_endpoints = manifest.clone();
    unsorted_endpoints.source_endpoints.reverse();
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&unsorted_endpoints, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut after_completion = manifest.clone();
    let mut extra = after_completion.batches[1].clone();
    extra.ordinal = 2;
    extra.requested_version = manifest.selected_version;
    extra.first_version = None;
    extra.last_version = None;
    extra.applied_through_version = manifest.selected_version;
    extra.certified_latest_version = manifest.selected_version;
    after_completion.batch_count += 1;
    after_completion.total_report_bytes += extra.report_bytes;
    after_completion.query_call_count += extra.query_call_count;
    after_completion.response_bytes += extra.response_bytes;
    after_completion.applied_mutation_count += extra.applied_mutation_count;
    after_completion.batches.push(extra);
    let three_batch_limits = NnsCertifiedRegistryArchiveLimits::new(
        3,
        limits.max_batch_report_bytes,
        limits.max_total_report_bytes * 2,
    );
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&after_completion, three_batch_limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let too_small = NnsCertifiedRegistryArchiveLimits::new(
        1,
        limits.max_batch_report_bytes,
        limits.max_total_report_bytes,
    );
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&manifest, too_small),
        Err(NnsCertifiedRegistryArchiveError::LimitExceeded {
            field: "batch count",
            maximum: 1,
            actual: 2,
        })
    ));
}

#[test]
fn confined_archive_publication_and_sequential_restore_round_trip() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-round-trip");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let storage_limits = archive_storage_limits();
    let mut archive_writer = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
    );

    archive_writer
        .apply_batch(&first)
        .expect("first archive object");
    assert!(
        !nns_certified_registry_archive_manifest_path(&archive_root).exists(),
        "partial publication has no discoverable manifest"
    );
    archive_writer
        .apply_batch(&second)
        .expect("second archive object");
    let archive = archive_writer.finish().expect("atomic archive manifest");

    assert_eq!(archive.manifest().batch_count, 2);
    assert_eq!(
        archive.replay_session().replay_session().selected_version(),
        Some(2)
    );
    let object_paths = fs::read_dir(archive_root.join("objects"))
        .expect("archive objects directory")
        .map(|entry| entry.expect("archive object entry").path())
        .collect::<Vec<_>>();
    assert_eq!(object_paths.len(), 2);
    assert!(object_paths.iter().all(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    }));

    let restored = super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
        &FixtureArchiveAuthenticator,
    )
    .expect("bounded sequential archive restoration");

    assert_eq!(restored.manifest(), archive.manifest());
    assert_eq!(
        restored
            .replay_session()
            .replay_session()
            .state()
            .get(b"a")
            .expect("restored first value")
            .value(),
        b"one"
    );
    assert_eq!(
        restored
            .replay_session()
            .replay_session()
            .state()
            .get(b"b")
            .expect("restored second value")
            .value(),
        b"two"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_archive_manifest_publication_preserves_prior_complete_archive() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-preserve");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let storage_limits = archive_storage_limits();
    let mut initial = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
    );
    initial.apply_batch(&first).expect("initial first object");
    initial.apply_batch(&second).expect("initial second object");
    initial.finish().expect("initial complete archive");
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);
    let original_manifest = fs::read(&manifest_path).expect("original archive manifest");

    let tiny_manifest_limits =
        NnsCertifiedRegistryArchiveStorageLimits::new(1, storage_limits.archive);
    let mut replacement = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        fixture.limits,
        tiny_manifest_limits,
    );
    replacement
        .apply_batch(&first)
        .expect("replacement first object");
    replacement
        .apply_batch(&second)
        .expect("replacement second object");
    let error = replacement
        .finish()
        .expect_err("oversized replacement manifest");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveStorageError::FileLimitExceeded {
            kind: "manifest",
            maximum: 1,
            ..
        }
    ));
    assert_eq!(
        fs::read(&manifest_path).expect("preserved archive manifest"),
        original_manifest
    );
    let restored = super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
        &FixtureArchiveAuthenticator,
    )
    .expect("prior complete archive remains restorable");
    assert_eq!(restored.manifest().batch_count, 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn archive_restore_rejects_tampered_objects_before_authentication() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-tamper");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let storage_limits = archive_storage_limits();
    let mut archive_writer = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
    );
    archive_writer
        .apply_batch(&first)
        .expect("first archive object");
    archive_writer
        .apply_batch(&second)
        .expect("second archive object");
    let archive = archive_writer.finish().expect("complete archive");
    let first_object = archive_root.join("objects").join(format!(
        "{}.json",
        archive.manifest().batches[0].report_sha256
    ));
    let mut tampered = fs::read_to_string(&first_object).expect("first archive object text");
    tampered.replace_range(..1, "[");
    crate::cache_file::write_managed_text_atomically(&root, &first_object, &tampered)
        .expect("replace object with same-length tampered content");

    let error = super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
        &PanicArchiveAuthenticator,
    )
    .expect_err("tampered object digest");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveStorageError::BatchDigestMismatch { ordinal: 0, .. }
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn archive_restore_bounds_manifest_and_rejects_missing_or_noncanonical_files() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-load-errors");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let storage_limits = archive_storage_limits();
    let mut archive_writer = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        fixture.limits,
        storage_limits,
    );
    archive_writer
        .apply_batch(&first)
        .expect("first archive object");
    archive_writer
        .apply_batch(&second)
        .expect("second archive object");
    let archive = archive_writer.finish().expect("complete archive");
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);

    let tiny_manifest_limits =
        NnsCertifiedRegistryArchiveStorageLimits::new(1, storage_limits.archive);
    let limit_error =
        super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
            &root,
            &archive_root,
            fixture.limits,
            tiny_manifest_limits,
            &PanicArchiveAuthenticator,
        )
        .expect_err("manifest metadata exceeds read ceiling");
    assert!(matches!(
        limit_error,
        NnsCertifiedRegistryArchiveStorageError::FileLimitExceeded {
            kind: "manifest",
            maximum: 1,
            ..
        }
    ));

    let first_object = archive_root.join("objects").join(format!(
        "{}.json",
        archive.manifest().batches[0].report_sha256
    ));
    fs::remove_file(&first_object).expect("remove first object fixture");
    let missing_error =
        super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
            &root,
            &archive_root,
            fixture.limits,
            storage_limits,
            &PanicArchiveAuthenticator,
        )
        .expect_err("manifest-referenced object is mandatory");
    assert!(matches!(
        missing_error,
        NnsCertifiedRegistryArchiveStorageError::MissingBatchObject { ordinal: 0, .. }
    ));

    crate::cache_file::write_managed_text_atomically(
        &root,
        &first_object,
        &serde_json::to_string(&fixture.first).expect("canonical first report"),
    )
    .expect("restore first object fixture");
    let mut noncanonical = fs::read_to_string(&manifest_path).expect("canonical manifest");
    noncanonical.push('\n');
    crate::cache_file::write_managed_text_atomically(&root, &manifest_path, &noncanonical)
        .expect("publish noncanonical manifest fixture");
    let canonical_error =
        super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
            &root,
            &archive_root,
            fixture.limits,
            storage_limits,
            &PanicArchiveAuthenticator,
        )
        .expect_err("manifest encoding must be canonical");
    assert!(matches!(
        canonical_error,
        NnsCertifiedRegistryArchiveStorageError::NonCanonicalManifest { .. }
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn archive_publication_rejects_unconfined_paths_and_poisoned_reuse() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-confined");
    let outside = crate::test_support::temp_dir("ic-query-registry-archive-outside");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let mut publisher = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &outside,
        fixture.limits,
        archive_storage_limits(),
    );

    let error = publisher
        .apply_batch(&first)
        .expect_err("archive root outside capability root");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveStorageError::FileOperation {
            source: crate::cache_file::CacheFileError::Confinement { .. },
        }
    ));
    assert!(matches!(
        publisher.apply_batch(&first),
        Err(NnsCertifiedRegistryArchiveStorageError::PublisherPoisoned)
    ));
    assert!(!outside.exists());
    let _ = fs::remove_dir_all(root);
}

struct FixtureArchiveAuthenticator;

impl super::archive::storage::ArchiveBatchAuthenticator for FixtureArchiveAuthenticator {
    fn authenticate<'a>(
        &self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        validate_nns_certified_registry_delta_batch(request, report)?;
        Ok(NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(
            report,
        ))
    }
}

struct PanicArchiveAuthenticator;

impl super::archive::storage::ArchiveBatchAuthenticator for PanicArchiveAuthenticator {
    fn authenticate<'a>(
        &self,
        _request: &NnsCertifiedRegistryDeltaBatchRequest,
        _report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        panic!("object tampering must fail before authentication")
    }
}

const fn archive_storage_limits() -> NnsCertifiedRegistryArchiveStorageLimits {
    NnsCertifiedRegistryArchiveStorageLimits::new(
        100_000,
        NnsCertifiedRegistryArchiveLimits::new(2, 100_000, 200_000),
    )
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

struct ProvenanceFixture {
    limits: NnsRegistryReplaySessionLimits,
    first_request: NnsCertifiedRegistryDeltaBatchRequest,
    first: NnsCertifiedRegistryDeltaBatchReport,
    second_request: NnsCertifiedRegistryDeltaBatchRequest,
    second: NnsCertifiedRegistryDeltaBatchReport,
}

fn provenance_fixture() -> ProvenanceFixture {
    let limits =
        NnsRegistryReplaySessionLimits::new(2, 2, 2, 128, NnsRegistryReplayLimits::new(10, 100));
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
    let mut second_request = request(1);
    second_request.source_endpoint = "https://example.com".to_string();
    let mut second = report(
        &second_request,
        2,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"b",
            Some(b"two"),
        )],
        Vec::new(),
    );
    second.certification.certificate_time_nanos = (NOW - 60) * 1_000_000_000;
    second.certification.certificate_time = format_utc_timestamp_secs(NOW - 60);
    ProvenanceFixture {
        limits,
        first_request,
        first,
        second_request,
        second,
    }
}

fn complete_provenance_session(changed_raw_evidence: bool) -> NnsRegistryReplaySession {
    let mut fixture = provenance_fixture();
    if changed_raw_evidence {
        fixture.second.certification.certificate_hex = "ce".repeat(8);
    }
    let mut session = NnsRegistryReplaySession::new(fixture.limits);
    session
        .apply_batch(&fixture.first_request, &fixture.first)
        .expect("first complete-session provenance batch");
    session
        .apply_batch(&fixture.second_request, &fixture.second)
        .expect("second complete-session provenance batch");
    session
}

fn projection_session() -> NnsRegistryReplaySession {
    NnsRegistryReplaySession::new(NnsRegistryReplaySessionLimits::new(
        1,
        1,
        1,
        64,
        NnsRegistryReplayLimits::new(20, 10_000),
    ))
}

fn complete_catalog_projection_session(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
) -> NnsRegistryReplaySession {
    complete_catalog_projection_session_from_parts(
        include_subnet_record,
        invalid_subnet_list,
        projection_routing_table_with_range(PROJECTION_SUBNET),
    )
}

fn complete_catalog_projection_session_with_routing(
    routing_table: RoutingTable,
) -> NnsRegistryReplaySession {
    complete_catalog_projection_session_from_parts(true, false, routing_table)
}

fn complete_catalog_projection_session_from_parts(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
    routing_table: RoutingTable,
) -> NnsRegistryReplaySession {
    let subnet_list = SubnetListRecord {
        subnets: vec![principal_bytes(PROJECTION_SUBNET)],
    };
    let subnet_record = SubnetRecord {
        membership: vec![
            principal_bytes("aaaaa-aa"),
            principal_bytes(PROJECTION_CANISTER),
        ],
        subnet_type: SubnetType::Application as i32,
        canister_cycles_cost_schedule: 0,
    };
    let mut records = vec![
        (b"routing_table".as_slice(), routing_table.encode_to_vec()),
        (
            b"subnet_list".as_slice(),
            if invalid_subnet_list {
                vec![0xff]
            } else {
                subnet_list.encode_to_vec()
            },
        ),
    ];
    let subnet_record_key = format!("subnet_record_{PROJECTION_SUBNET}");
    if include_subnet_record {
        records.push((subnet_record_key.as_bytes(), subnet_record.encode_to_vec()));
    }
    records.sort_by(|left, right| left.0.cmp(right.0));
    let mutations = records
        .into_iter()
        .map(|(key, value)| mutation(NnsCertifiedRegistryMutationKind::Upsert, key, Some(&value)))
        .collect();
    let request = request(0);
    let report = report(&request, 1, 1, mutations, Vec::new());
    let mut session = projection_session();
    session
        .apply_batch(&request, &report)
        .expect("complete catalog fixture replay");
    session
}

fn projection_routing_table_with_range(subnet: &str) -> RoutingTable {
    RoutingTable {
        entries: vec![RoutingTableEntry {
            range: Some(CanisterIdRange {
                start_canister_id: Some(canister_id(PROJECTION_CANISTER)),
                end_canister_id: Some(canister_id(PROJECTION_CANISTER)),
            }),
            subnet_id: Some(SubnetId {
                principal_id: Some(PrincipalId {
                    raw: principal_bytes(subnet),
                }),
            }),
        }],
    }
}

fn canister_id(principal: &str) -> CanisterId {
    CanisterId {
        principal_id: Some(PrincipalId {
            raw: principal_bytes(principal),
        }),
    }
}

fn principal_bytes(principal: &str) -> Vec<u8> {
    Principal::from_text(principal)
        .expect("projection fixture principal")
        .as_slice()
        .to_vec()
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
        schema_version: 3,
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
        chunk_evidence_bytes: 0,
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
        chunk_evidence: Vec::new(),
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
