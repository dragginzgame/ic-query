//! Module: nns::registry::replay::tests::projection
//!
//! Responsibility: authenticated Registry-state to Subnet-catalog projection tests.
//! Does not own: production replay behavior or shared protocol fixtures.
//! Boundary: exercises the corresponding replay subsystem through fixture evidence.

use super::*;

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
fn authenticated_archive_promotes_one_certified_catalog_authority() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-projection");
    let archive = complete_catalog_archive(&root);
    let request = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );

    let authority = project_nns_certified_subnet_catalog(&archive, &request)
        .expect("archive-backed certified catalog");
    let catalog = authority.catalog();
    let evidence = catalog
        .provenance()
        .certified_registry
        .as_ref()
        .expect("certified archive commitments");

    assert_eq!(authority.archive(), &archive);
    assert_eq!(catalog.raw().catalog_schema_version, CATALOG_SCHEMA_VERSION);
    assert_eq!(catalog.provenance().assurance, CatalogAssurance::Certified);
    assert_eq!(catalog.provenance().registry_version, 1);
    assert_eq!(catalog.subnets().len(), 1);
    assert_eq!(catalog.subnets()[0].subnet_principal, PROJECTION_SUBNET);
    assert_eq!(
        authority.freshness(),
        NnsCertifiedSubnetCatalogFreshness {
            observation_time_unix_seconds: NOW,
            latest_certificate_time_nanos: NOW * 1_000_000_000,
            certificate_age_nanos: 0,
            maximum_certificate_age_seconds: 0,
            selected_registry_version: 1,
            maximum_observed_certified_registry_version: 1,
            version_policy: NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
        }
    );
    assert_eq!(
        evidence.archive_manifest_schema_version,
        archive.manifest().schema_version
    );
    assert_eq!(
        evidence.evidence_chain_digest,
        archive.manifest().evidence_chain_digest
    );
    assert_eq!(
        evidence.complete_state_digest,
        archive.manifest().complete_state_digest
    );

    let serialized_claim = catalog_to_pretty_json(catalog.raw()).expect("serialize raw claim");
    let serialized_claim = parse_catalog_json(&serialized_claim).expect("parse raw claim");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(serialized_claim, &request.validation),
        Err(CatalogError::UnsupportedAssurance { assurance }) if assurance == "certified"
    ));

    let mut mismatched_claim = catalog.to_raw();
    mismatched_claim
        .provenance
        .certified_registry
        .as_mut()
        .expect("certified archive commitments")
        .complete_state_digest = "ff".repeat(32);
    mismatched_claim
        .canonicalize_and_seal()
        .expect("reseal structurally valid mismatched claim");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_authenticated_archive(
            mismatched_claim,
            &request.validation,
            &archive,
        ),
        Err(CatalogError::InvalidProvenance {
            field: "provenance",
            ..
        })
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_projection_enforces_explicit_certificate_age() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-freshness");
    let archive = complete_catalog_archive(&root);

    let boundary = certified_catalog_projection_request(
        NOW + 60,
        60,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    let authority = project_nns_certified_subnet_catalog(&archive, &boundary)
        .expect("certificate at the inclusive maximum age");
    assert_eq!(authority.freshness().certificate_age_nanos, 60_000_000_000);

    let stale = certified_catalog_projection_request(
        NOW + 61,
        60,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    assert!(matches!(
        project_nns_certified_subnet_catalog(&archive, &stale),
        Err(NnsRegistrySubnetCatalogProjectionError::StaleArchiveCertificate {
            latest_certificate_time_nanos,
            observation_time_unix_seconds,
            certificate_age_nanos: 61_000_000_000,
            maximum_certificate_age_seconds: 60,
        }) if latest_certificate_time_nanos == NOW * 1_000_000_000
            && observation_time_unix_seconds == NOW + 61
    ));

    let before_certificate = certified_catalog_projection_request(
        NOW - 1,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    assert!(matches!(
        project_nns_certified_subnet_catalog(&archive, &before_certificate),
        Err(NnsRegistrySubnetCatalogProjectionError::Catalog(
            CatalogError::FutureTimestamp { .. }
        ))
    ));

    let maximum_domain = certified_catalog_projection_request(
        u64::MAX,
        u64::MAX,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    let authority = project_nns_certified_subnet_catalog(&archive, &maximum_domain)
        .expect("full u64 freshness policy is representable");
    assert!(authority.freshness().certificate_age_nanos > u128::from(u64::MAX));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_projection_makes_known_version_lag_explicit() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-version-policy");
    let archive = superseded_catalog_archive(&root);
    assert_eq!(archive.manifest().selected_version, 2);
    assert_eq!(archive.manifest().batches[1].certified_latest_version, 3);

    let require_latest = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    assert!(matches!(
        project_nns_certified_subnet_catalog(&archive, &require_latest),
        Err(
            NnsRegistrySubnetCatalogProjectionError::SupersededArchiveTarget {
                selected_registry_version: 2,
                maximum_observed_certified_registry_version: 3,
            }
        )
    ));

    let allow_historical = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::AllowHistoricalTarget,
    );
    let authority = project_nns_certified_subnet_catalog(&archive, &allow_historical)
        .expect("explicit historical exact-target authority");
    assert_eq!(authority.catalog().provenance().registry_version, 2);
    assert_eq!(
        authority
            .freshness()
            .maximum_observed_certified_registry_version,
        3
    );
    assert_eq!(
        authority.freshness().version_policy,
        NnsCertifiedSubnetCatalogVersionPolicy::AllowHistoricalTarget
    );
    let _ = fs::remove_dir_all(root);
}
