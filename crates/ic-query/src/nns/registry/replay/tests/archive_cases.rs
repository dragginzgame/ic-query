//! Module: nns::registry::replay::tests::archive_cases
//!
//! Responsibility: authenticated archive model, storage, refresh, and cleanup tests.
//! Does not own: production replay behavior or shared protocol fixtures.
//! Boundary: exercises the corresponding replay subsystem through fixture evidence.

use super::*;

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
    assert_eq!(manifest.segment_count, 1);
    assert_eq!(manifest.batch_count, 2);
    assert_eq!(manifest.batches.len(), 2);
    assert_eq!(manifest.batches[0].ordinal, 0);
    assert_eq!(manifest.batches[0].segment_ordinal, 0);
    assert_eq!(manifest.batches[0].segment_target_version, 2);
    assert_eq!(manifest.batches[0].requested_version, 0);
    assert_eq!(manifest.batches[0].applied_through_version, 1);
    assert_eq!(manifest.batches[1].ordinal, 1);
    assert_eq!(manifest.batches[1].segment_ordinal, 0);
    assert_eq!(manifest.batches[1].segment_target_version, 2);
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
        "current manifests reject undeclared fields"
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
fn archive_manifest_segments_retain_unchanged_and_advancing_authenticated_targets() {
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let no_change_time = NOW + 60;
    let no_change_request = NnsCertifiedRegistryDeltaBatchRequest::new(
        MAINNET_NETWORK,
        "https://icp-api.io",
        2,
        no_change_time,
    );
    let mut no_change_report = report_versions(&no_change_request, 2, Vec::new());
    no_change_report.fetched_at = format_utc_timestamp_secs(no_change_time);
    no_change_report.certification.certificate_time_nanos = no_change_time * 1_000_000_000;
    no_change_report.certification.certificate_time = format_utc_timestamp_secs(no_change_time);
    let no_change = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&no_change_report);
    let advance_request = request(2);
    let advance_report = report_versions(
        &advance_request,
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
                    b"c",
                    Some(b"four"),
                )],
            ),
        ],
    );
    let advance = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&advance_report);
    let replay_limits = extended_replay_limits();
    let archive_limits = extended_archive_storage_limits().archive;
    let mut builder =
        NnsCertifiedRegistryArchiveManifestBuilder::new(replay_limits, archive_limits);

    builder.apply_batch(&first).expect("first bootstrap batch");
    builder
        .apply_batch(&second)
        .expect("complete bootstrap segment");
    let state_digest = builder
        .replay_session()
        .complete_state_digest()
        .expect("bootstrap state digest");
    let no_change_progress = builder
        .apply_batch(&no_change)
        .expect("fresh unchanged-version segment");
    assert_eq!(no_change_progress.through_version, 2);
    assert_eq!(no_change_progress.applied_version_count, 0);
    assert_eq!(
        builder.replay_session().complete_state_digest(),
        Some(state_digest)
    );
    builder
        .apply_batch(&advance)
        .expect("advancing exact-target segment");

    let (manifest, authenticated) = builder.finish().expect("segmented archive manifest");

    assert_eq!(manifest.schema_version, 2);
    assert_eq!(manifest.segment_count, 3);
    assert_eq!(manifest.selected_version, 4);
    assert_eq!(manifest.batch_count, 4);
    assert_eq!(
        manifest.maximum_certificate_time_nanos,
        no_change_time * 1_000_000_000
    );
    assert_eq!(
        manifest
            .batches
            .iter()
            .map(|batch| (batch.segment_ordinal, batch.segment_target_version))
            .collect::<Vec<_>>(),
        vec![(0, 2), (0, 2), (1, 2), (2, 4)]
    );
    assert_eq!(
        authenticated
            .replay_session()
            .state()
            .get(b"a")
            .expect("a")
            .value(),
        b"three"
    );
    assert_eq!(
        authenticated
            .replay_session()
            .state()
            .get(b"c")
            .expect("c")
            .value(),
        b"four"
    );
    validate_nns_certified_registry_archive_manifest(&manifest, archive_limits)
        .expect("segmented manifest validates");
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

    let mut changed_segment_count = manifest.clone();
    changed_segment_count.segment_count += 1;
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&changed_segment_count, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut changed_segment_target = manifest.clone();
    changed_segment_target.batches[1].segment_target_version += 1;
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&changed_segment_target, limits),
        Err(NnsCertifiedRegistryArchiveError::InvalidManifest { .. })
    ));

    let mut unsorted_endpoints = manifest.clone();
    unsorted_endpoints.source_endpoints.reverse();
    assert!(matches!(
        validate_nns_certified_registry_archive_manifest(&unsorted_endpoints, limits),
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

    let restored =
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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
fn confined_archive_publisher_resumes_reauthenticated_state_without_rewriting_history() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-resume");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let fixture = provenance_fixture();
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.first);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&fixture.second);
    let replay_limits = extended_replay_limits();
    let storage_limits = extended_archive_storage_limits();
    let mut initial = NnsCertifiedRegistryArchivePublisher::new(
        &root,
        &archive_root,
        replay_limits,
        storage_limits,
    );
    initial.apply_batch(&first).expect("initial first object");
    initial.apply_batch(&second).expect("initial second object");
    let initial = initial.finish().expect("initial archive");
    let initial_digests = initial
        .manifest()
        .batches
        .iter()
        .map(|batch| batch.report_sha256.clone())
        .collect::<Vec<_>>();
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);
    let initial_manifest_bytes = fs::read(&manifest_path).expect("initial manifest bytes");

    let extension_request = request(2);
    let extension_report = report(
        &extension_request,
        3,
        3,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Update,
            b"a",
            Some(b"extended"),
        )],
        Vec::new(),
    );
    let extension = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&extension_report);

    assert_constrained_resume_preserves_manifest(
        &root,
        &archive_root,
        storage_limits,
        &extension,
        &manifest_path,
        &initial_manifest_bytes,
    );

    let mut resumed = super::super::archive::storage::resume_archive_publisher_with_authenticator(
        &root,
        &archive_root,
        replay_limits,
        storage_limits,
        &FixtureArchiveAuthenticator,
    )
    .expect("reauthenticated resumable publisher");
    assert_eq!(resumed.replay_session().state().through_version(), 2);
    resumed
        .apply_batch(&extension)
        .expect("durable extension object");
    let extended = resumed.finish().expect("extended archive manifest");

    assert_eq!(extended.manifest().schema_version, 2);
    assert_eq!(extended.manifest().segment_count, 2);
    assert_eq!(extended.manifest().selected_version, 3);
    assert_eq!(extended.manifest().batch_count, 3);
    assert_eq!(
        extended
            .manifest()
            .batches
            .iter()
            .take(2)
            .map(|batch| batch.report_sha256.clone())
            .collect::<Vec<_>>(),
        initial_digests
    );
    let object_count = fs::read_dir(archive_root.join("objects"))
        .expect("extended archive objects")
        .count();
    assert_eq!(object_count, 3);

    let restored =
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
            &root,
            &archive_root,
            replay_limits,
            storage_limits,
            &FixtureArchiveAuthenticator,
        )
        .expect("extended archive reloads");
    assert_eq!(restored, extended);
    assert_eq!(
        restored
            .replay_session()
            .replay_session()
            .state()
            .get(b"a")
            .expect("extended a")
            .value(),
        b"extended"
    );
    let _ = fs::remove_dir_all(root);
}

fn assert_constrained_resume_preserves_manifest(
    root: &Path,
    archive_root: &Path,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    extension: &NnsAuthenticatedRegistryDeltaBatch<'_>,
    manifest_path: &Path,
    initial_manifest_bytes: &[u8],
) {
    let constrained_replay_limits = NnsRegistryReplaySessionLimits::new(
        10,
        2,
        130,
        80 * 1_024 * 1_024,
        NnsRegistryReplayLimits::new(20, 1_000),
    );
    let mut constrained =
        super::super::archive::storage::resume_archive_publisher_with_authenticator(
            root,
            archive_root,
            constrained_replay_limits,
            storage_limits,
            &FixtureArchiveAuthenticator,
        )
        .expect("existing archive fits exact cumulative limits");
    let error = constrained
        .apply_batch(extension)
        .expect_err("extension exceeds cumulative batch limit");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveStorageError::Archive(NnsCertifiedRegistryArchiveError::Replay(
            NnsRegistryReplayError::SessionLimitExceeded {
                field: "batch count",
                maximum: 2,
                actual: 3,
            }
        ))
    ));
    assert_eq!(
        fs::read(manifest_path).expect("preserved initial manifest"),
        initial_manifest_bytes
    );
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
    let restored =
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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

    let error =
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
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

impl super::super::archive::storage::ArchiveBatchAuthenticator for FixtureArchiveAuthenticator {
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

impl super::super::archive::storage::ArchiveBatchAuthenticator for PanicArchiveAuthenticator {
    fn authenticate<'a>(
        &self,
        _request: &NnsCertifiedRegistryDeltaBatchRequest,
        _report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        panic!("object tampering must fail before authentication")
    }
}

struct LockCheckingArchiveAuthenticator {
    lock_path: PathBuf,
}

impl super::super::archive::storage::ArchiveBatchAuthenticator
    for LockCheckingArchiveAuthenticator
{
    fn authenticate<'a>(
        &self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        assert!(
            self.lock_path.is_file(),
            "archive authentication must run under the maintenance lock"
        );
        FixtureArchiveAuthenticator.authenticate(request, report)
    }
}

#[derive(Default)]
struct RejectExtensionAuthenticator {
    accepted_reports: Mutex<usize>,
}

impl super::super::archive::storage::ArchiveBatchAuthenticator for RejectExtensionAuthenticator {
    fn authenticate<'a>(
        &self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &'a NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsAuthenticatedRegistryDeltaBatch<'a>, NnsRegistryHostError> {
        let mut accepted_reports = self
            .accepted_reports
            .lock()
            .expect("reject-extension authenticator lock");
        if *accepted_reports == 2 {
            return Err(NnsRegistryHostError::InvalidSourceData {
                reason: "fixture rejects the extension report".to_string(),
            });
        }
        *accepted_reports += 1;
        drop(accepted_reports);
        validate_nns_certified_registry_delta_batch(request, report)?;
        Ok(NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(
            report,
        ))
    }
}

const fn archive_storage_limits() -> NnsCertifiedRegistryArchiveStorageLimits {
    NnsCertifiedRegistryArchiveStorageLimits::new(
        100_000,
        NnsCertifiedRegistryArchiveLimits::new(2, 100_000, 200_000),
    )
}

const fn extended_replay_limits() -> NnsRegistryReplaySessionLimits {
    NnsRegistryReplaySessionLimits::new(
        10,
        8,
        520,
        320 * 1_024 * 1_024,
        NnsRegistryReplayLimits::new(20, 1_000),
    )
}

const fn extended_archive_storage_limits() -> NnsCertifiedRegistryArchiveStorageLimits {
    NnsCertifiedRegistryArchiveStorageLimits::new(
        500_000,
        NnsCertifiedRegistryArchiveLimits::new(8, 100_000, 800_000),
    )
}

fn fixture_archive_bootstrap(
    request: &NnsCertifiedRegistryArchiveBootstrapRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveBootstrapError> {
    futures::executor::block_on(
        super::super::archive::bootstrap_archive_with_authenticator_async(
            request,
            source,
            &FixtureArchiveAuthenticator,
        ),
    )
}

fn fixture_archive_refresh(
    request: &NnsCertifiedRegistryArchiveRefreshRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveRefreshError> {
    futures::executor::block_on(
        super::super::archive::refresh_archive_with_authenticator_async(
            request,
            source,
            &FixtureArchiveAuthenticator,
        ),
    )
}

fn bootstrap_fixture_archive(
    cache_root: &Path,
    archive_root: &Path,
    replay_limits: NnsRegistryReplaySessionLimits,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
) -> NnsAuthenticatedRegistryArchive {
    let collection = NnsCertifiedRegistryBootstrapRequest::new(
        MAINNET_NETWORK,
        "https://icp-api.io",
        NOW,
        replay_limits,
    );
    let request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        collection,
        cache_root,
        archive_root,
        storage_limits,
        300,
    );
    fixture_archive_bootstrap(&request, &BootstrapSource::default())
        .expect("fixture archive bootstrap")
}

fn archive_refresh_request(
    network: &str,
    cache_root: &Path,
    archive_root: &Path,
    replay_limits: NnsRegistryReplaySessionLimits,
    storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
) -> NnsCertifiedRegistryArchiveRefreshRequest {
    NnsCertifiedRegistryArchiveRefreshRequest::new(
        NnsCertifiedRegistryBootstrapRequest::new(
            network,
            "https://icp-api.io",
            NOW,
            replay_limits,
        ),
        cache_root,
        archive_root,
        storage_limits,
        300,
    )
}

fn archive_cleanup_request(
    cache_root: &Path,
    archive_root: &Path,
    cleanup_limits: NnsCertifiedRegistryArchiveCleanupLimits,
) -> NnsCertifiedRegistryArchiveCleanupRequest {
    NnsCertifiedRegistryArchiveCleanupRequest::new(
        NOW,
        cache_root,
        archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
        cleanup_limits,
        300,
    )
}

fn write_archive_orphan(cache_root: &Path, archive_root: &Path, name: &str, contents: &[u8]) {
    let path = archive_root.join("objects").join(name);
    crate::cache_file::write_managed_file_atomically(cache_root, &path, |file| {
        std::io::Write::write_all(file, contents)
    })
    .expect("orphan fixture write");
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
fn certified_archive_bootstrap_streams_one_locked_complete_archive() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-bootstrap");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let source = BootstrapSource::default();
    let request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap_request(MAINNET_NETWORK, 2, 130, 80 * 1_024 * 1_024),
        &root,
        &archive_root,
        archive_storage_limits(),
        300,
    );

    let archive = fixture_archive_bootstrap(&request, &source).expect("bounded archive bootstrap");

    assert_eq!(source.requested_versions(), vec![0, 2]);
    assert_eq!(archive.manifest().selected_version, 3);
    assert_eq!(archive.manifest().batch_count, 2);
    assert_eq!(
        archive
            .replay_session()
            .replay_session()
            .state()
            .through_version(),
        3
    );
    assert!(nns_certified_registry_archive_manifest_path(&archive_root).is_file());
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());

    let loaded =
        super::super::archive::storage::load_nns_certified_registry_archive_with_authenticator(
            &root,
            &archive_root,
            request.bootstrap.limits,
            request.storage_limits,
            &FixtureArchiveAuthenticator,
        )
        .expect("published archive reloads from retained fixture evidence");
    assert_eq!(loaded, archive);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_bootstrap_rejects_non_mainnet_before_filesystem_or_source_work() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-bootstrap-network");
    let archive_root = root.join("nns/local/registry-certified-v1");
    let source = BootstrapSource::default();
    let request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap_request("local", 2, 130, 80 * 1_024 * 1_024),
        &root,
        &archive_root,
        archive_storage_limits(),
        300,
    );

    let error =
        fixture_archive_bootstrap(&request, &source).expect_err("non-mainnet archive bootstrap");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveBootstrapError::Replay(
            NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork {
                network
            })
        ) if network == "local"
    ));
    assert!(source.requested_versions().is_empty());
    assert!(!archive_root.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_bootstrap_reauthenticates_custom_source_reports_before_publication() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-bootstrap-auth");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let source = BootstrapSource::default();
    let request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap_request(MAINNET_NETWORK, 2, 130, 80 * 1_024 * 1_024),
        &root,
        &archive_root,
        archive_storage_limits(),
        300,
    );

    let error = futures::executor::block_on(
        bootstrap_nns_certified_registry_archive_with_source_async(&request, &source),
    )
    .expect_err("fixture certificate cannot establish archive authority");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveBootstrapError::BatchAuthentication {
            requested_version: 0,
            ..
        }
    ));
    assert_eq!(source.requested_versions(), vec![0]);
    assert!(!nns_certified_registry_archive_manifest_path(&archive_root).exists());
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_bootstrap_reserves_before_each_source_call() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-bootstrap-capacity");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let source = BootstrapSource::default();
    let request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap_request(MAINNET_NETWORK, 1, 65, 40 * 1_024 * 1_024),
        &root,
        &archive_root,
        archive_storage_limits(),
        300,
    );

    let error = fixture_archive_bootstrap(&request, &source)
        .expect_err("second archive batch lacks worst-case reservation");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveBootstrapError::Replay(
            NnsRegistryReplayError::SessionLimitExceeded {
                field: "batch count",
                maximum: 1,
                actual: 2,
            }
        )
    ));
    assert_eq!(source.requested_versions(), vec![0]);
    assert!(!nns_certified_registry_archive_manifest_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_archive_force_bootstrap_preserves_the_previous_complete_manifest() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-bootstrap-atomic");
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let bootstrap = bootstrap_request(MAINNET_NETWORK, 2, 130, 80 * 1_024 * 1_024);
    let initial_request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap.clone(),
        &root,
        &archive_root,
        archive_storage_limits(),
        300,
    );
    fixture_archive_bootstrap(&initial_request, &BootstrapSource::default())
        .expect("initial complete archive");
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);
    let before = fs::read(&manifest_path).expect("initial manifest bytes");

    let constrained_storage = NnsCertifiedRegistryArchiveStorageLimits::new(
        100_000,
        NnsCertifiedRegistryArchiveLimits::new(1, 100_000, 200_000),
    );
    let replacement_request = NnsCertifiedRegistryArchiveBootstrapRequest::new(
        bootstrap,
        &root,
        &archive_root,
        constrained_storage,
        300,
    );
    let source = BootstrapSource::default();
    let error = fixture_archive_bootstrap(&replacement_request, &source)
        .expect_err("replacement archive exceeds its explicit storage ceiling");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveBootstrapError::Storage(
            NnsCertifiedRegistryArchiveStorageError::Archive(
                NnsCertifiedRegistryArchiveError::LimitExceeded {
                    field: "batch count",
                    maximum: 1,
                    actual: 2,
                }
            )
        )
    ));
    assert_eq!(source.requested_versions(), vec![0]);
    assert_eq!(
        fs::read(&manifest_path).expect("preserved manifest"),
        before
    );
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_refresh_extends_one_exact_target_under_the_archive_lock() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-refresh");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let replay_limits = extended_replay_limits();
    let storage_limits = extended_archive_storage_limits();
    bootstrap_fixture_archive(&root, &archive_root, replay_limits, storage_limits);
    let source = ArchiveRefreshSource::new(
        ArchiveRefreshMode::Advancing,
        nns_certified_registry_archive_refresh_lock_path(&archive_root),
    );
    let request = archive_refresh_request(
        MAINNET_NETWORK,
        &root,
        &archive_root,
        replay_limits,
        storage_limits,
    );

    let archive = fixture_archive_refresh(&request, &source).expect("advancing archive refresh");

    assert_eq!(source.requested_versions(), vec![3, 4]);
    assert_eq!(archive.manifest().segment_count, 2);
    assert_eq!(archive.manifest().selected_version, 5);
    assert_eq!(archive.manifest().batch_count, 4);
    assert_eq!(archive.manifest().batches[2].segment_target_version, 5);
    assert_eq!(archive.manifest().batches[3].segment_target_version, 5);
    let state = archive.replay_session().replay_session().state();
    assert_eq!(state.through_version(), 5);
    assert_eq!(state.get(b"a").expect("updated a").value(), b"four");
    assert_eq!(state.get(b"b").expect("inserted b").value(), b"five");
    assert!(state.get(b"future").is_none());
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_refresh_retains_an_authenticated_unchanged_version_observation() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-refresh-unchanged");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let replay_limits = extended_replay_limits();
    let storage_limits = extended_archive_storage_limits();
    let initial = bootstrap_fixture_archive(&root, &archive_root, replay_limits, storage_limits);
    let source = ArchiveRefreshSource::new(
        ArchiveRefreshMode::Unchanged,
        nns_certified_registry_archive_refresh_lock_path(&archive_root),
    );
    let request = archive_refresh_request(
        MAINNET_NETWORK,
        &root,
        &archive_root,
        replay_limits,
        storage_limits,
    );

    let refreshed = fixture_archive_refresh(&request, &source).expect("unchanged archive refresh");

    assert_eq!(source.requested_versions(), vec![3]);
    assert_eq!(refreshed.manifest().segment_count, 2);
    assert_eq!(refreshed.manifest().selected_version, 3);
    assert_eq!(refreshed.manifest().batch_count, 3);
    assert_eq!(refreshed.manifest().batches[2].applied_mutation_count, 0);
    assert_eq!(
        refreshed.manifest().complete_state_digest,
        initial.manifest().complete_state_digest
    );
    assert_ne!(
        refreshed.manifest().evidence_chain_digest,
        initial.manifest().evidence_chain_digest
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_refresh_reserves_cumulative_capacity_before_source_work() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-refresh-capacity");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let storage_limits = extended_archive_storage_limits();
    bootstrap_fixture_archive(
        &root,
        &archive_root,
        extended_replay_limits(),
        storage_limits,
    );
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);
    let before = fs::read(&manifest_path).expect("initial manifest bytes");
    let constrained = NnsRegistryReplaySessionLimits::new(
        10,
        2,
        130,
        80 * 1_024 * 1_024,
        NnsRegistryReplayLimits::new(20, 1_000),
    );
    let source = ArchiveRefreshSource::new(
        ArchiveRefreshMode::Unchanged,
        nns_certified_registry_archive_refresh_lock_path(&archive_root),
    );
    let request = archive_refresh_request(
        MAINNET_NETWORK,
        &root,
        &archive_root,
        constrained,
        storage_limits,
    );

    let error = fixture_archive_refresh(&request, &source)
        .expect_err("extension lacks cumulative batch capacity");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveRefreshError::Replay(
            NnsRegistryReplayError::SessionLimitExceeded {
                field: "batch count",
                maximum: 2,
                actual: 3,
            }
        )
    ));
    assert!(source.requested_versions().is_empty());
    assert_eq!(fs::read(manifest_path).expect("preserved manifest"), before);
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_refresh_authentication_failure_preserves_the_prior_manifest() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-refresh-auth");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let replay_limits = extended_replay_limits();
    let storage_limits = extended_archive_storage_limits();
    bootstrap_fixture_archive(&root, &archive_root, replay_limits, storage_limits);
    let manifest_path = nns_certified_registry_archive_manifest_path(&archive_root);
    let before = fs::read(&manifest_path).expect("initial manifest bytes");
    let source = ArchiveRefreshSource::new(
        ArchiveRefreshMode::Unchanged,
        nns_certified_registry_archive_refresh_lock_path(&archive_root),
    );
    let request = archive_refresh_request(
        MAINNET_NETWORK,
        &root,
        &archive_root,
        replay_limits,
        storage_limits,
    );

    let error = futures::executor::block_on(
        super::super::archive::refresh_archive_with_authenticator_async(
            &request,
            &source,
            &RejectExtensionAuthenticator::default(),
        ),
    )
    .expect_err("new source report fails local authentication");

    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveRefreshError::BatchAuthentication {
            requested_version: 3,
            ..
        }
    ));
    assert_eq!(source.requested_versions(), vec![3]);
    assert_eq!(fs::read(manifest_path).expect("preserved manifest"), before);
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_refresh_rejects_non_mainnet_and_missing_archives_without_source_work() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-refresh-preflight");
    let archive_root = root.join("missing/nns-registry");
    let source = ArchiveRefreshSource::new(
        ArchiveRefreshMode::Unchanged,
        nns_certified_registry_archive_refresh_lock_path(&archive_root),
    );
    let local = archive_refresh_request(
        "local",
        &root,
        &archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
    );

    let error = fixture_archive_refresh(&local, &source).expect_err("non-mainnet refresh");
    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveRefreshError::Replay(
            NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork {
                network
            })
        ) if network == "local"
    ));
    assert!(!root.exists());

    let mainnet = archive_refresh_request(
        MAINNET_NETWORK,
        &root,
        &archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
    );
    let error = fixture_archive_refresh(&mainnet, &source).expect_err("missing archive refresh");
    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveRefreshError::Storage(
            NnsCertifiedRegistryArchiveStorageError::MissingManifest { .. }
        )
    ));
    assert!(source.requested_versions().is_empty());
    assert!(!root.exists());
}

#[test]
fn certified_archive_cleanup_removes_only_bounded_unreferenced_objects_under_lock() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-cleanup");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    let initial = bootstrap_fixture_archive(
        &root,
        &archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
    );
    write_archive_orphan(&root, &archive_root, "abandoned.json", b"orphan");
    write_archive_orphan(&root, &archive_root, "abandoned.json.tmp.10.20.30", b"temp");
    let request = archive_cleanup_request(
        &root,
        &archive_root,
        NnsCertifiedRegistryArchiveCleanupLimits::new(4, 2, 10),
    );
    let authenticator = LockCheckingArchiveAuthenticator {
        lock_path: nns_certified_registry_archive_refresh_lock_path(&archive_root),
    };

    let report =
        super::super::archive::cleanup_archive_with_authenticator(&request, &authenticator)
            .expect("bounded orphan cleanup");

    assert_eq!(report.archive, initial);
    assert_eq!(report.scanned_object_count, 4);
    assert_eq!(report.referenced_object_count, 2);
    assert_eq!(report.removed_object_count, 2);
    assert_eq!(report.removed_bytes, 10);
    assert!(!archive_root.join("objects/abandoned.json").exists());
    assert!(
        !archive_root
            .join("objects/abandoned.json.tmp.10.20.30")
            .exists()
    );
    for batch in &report.archive.manifest().batches {
        assert!(
            archive_root
                .join("objects")
                .join(format!("{}.json", batch.report_sha256))
                .is_file()
        );
    }
    assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_cleanup_applies_every_ceiling_before_deleting_an_orphan() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-cleanup-limits");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    bootstrap_fixture_archive(
        &root,
        &archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
    );
    let orphan_path = archive_root.join("objects/orphan.json");
    write_archive_orphan(&root, &archive_root, "orphan.json", b"orphan");
    let cases = [
        (
            NnsCertifiedRegistryArchiveCleanupLimits::new(2, 1, 6),
            "scanned object count",
            2,
            3,
        ),
        (
            NnsCertifiedRegistryArchiveCleanupLimits::new(3, 0, 6),
            "removed object count",
            0,
            1,
        ),
        (
            NnsCertifiedRegistryArchiveCleanupLimits::new(3, 1, 5),
            "removed object bytes",
            5,
            6,
        ),
    ];

    for (limits, field, maximum, actual) in cases {
        let request = archive_cleanup_request(&root, &archive_root, limits);
        let error = super::super::archive::cleanup_archive_with_authenticator(
            &request,
            &FixtureArchiveAuthenticator,
        )
        .expect_err("cleanup limit blocks deletion");

        assert!(matches!(
            error,
            NnsCertifiedRegistryArchiveCleanupError::LimitExceeded {
                field: actual_field,
                maximum: actual_maximum,
                actual: actual_value,
            } if actual_field == field && actual_maximum == maximum && actual_value == actual
        ));
        assert!(orphan_path.is_file());
        assert!(!nns_certified_registry_archive_refresh_lock_path(&archive_root).exists());
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_archive_cleanup_authenticates_before_deletion_and_requires_a_manifest() {
    let root = crate::test_support::temp_dir("ic-query-registry-archive-cleanup-auth");
    let archive_root = root.join("nns/ic/registry-certified-v2");
    bootstrap_fixture_archive(
        &root,
        &archive_root,
        extended_replay_limits(),
        extended_archive_storage_limits(),
    );
    let orphan_path = archive_root.join("objects/orphan.json");
    write_archive_orphan(&root, &archive_root, "orphan.json", b"orphan");
    let request = archive_cleanup_request(
        &root,
        &archive_root,
        NnsCertifiedRegistryArchiveCleanupLimits::new(3, 1, 6),
    );

    let error = cleanup_nns_certified_registry_archive(&request)
        .expect_err("fixture archive is not built-in authenticated");
    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveCleanupError::Storage(
            NnsCertifiedRegistryArchiveStorageError::BatchAuthentication { ordinal: 0, .. }
        )
    ));
    assert!(orphan_path.is_file());

    let missing_root = crate::test_support::temp_dir("ic-query-registry-archive-cleanup-missing");
    let missing_archive = missing_root.join("missing/archive");
    let missing_request = archive_cleanup_request(
        &missing_root,
        &missing_archive,
        NnsCertifiedRegistryArchiveCleanupLimits::new(0, 0, 0),
    );
    let error = cleanup_nns_certified_registry_archive(&missing_request)
        .expect_err("cleanup requires an existing manifest");
    assert!(matches!(
        error,
        NnsCertifiedRegistryArchiveCleanupError::Storage(
            NnsCertifiedRegistryArchiveStorageError::MissingManifest { .. }
        )
    ));
    assert!(!missing_root.exists());
    let _ = fs::remove_dir_all(root);
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
