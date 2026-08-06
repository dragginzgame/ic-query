use super::{fixtures::*, *};

fn validation_context() -> CatalogValidationContext {
    CatalogValidationContext::new(
        MAINNET_NETWORK,
        MAINNET_REGISTRY_CANISTER_ID,
        1_780_531_300,
        DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
    )
}

#[test]
fn validated_route_retains_exact_catalog_authority() {
    let raw = fixture_catalog();
    let expected_digest = raw.catalog_digest.clone();
    let validated =
        ValidatedSubnetCatalog::try_from_raw(raw, &validation_context()).expect("valid catalog");

    let route = validated
        .resolve_canister_route(CANISTER_A)
        .expect("canister route");

    assert_eq!(route.canister.to_text(), CANISTER_A);
    assert_eq!(route.subnet.to_text(), SUBNET_A);
    assert_eq!(route.subnet_info.subnet_principal, SUBNET_A);
    assert_eq!(route.subnet_info.subnet_kind, SubnetKind::Application);
    assert_eq!(route.registry_version, 123_456);
    assert_eq!(
        route.provenance.assurance,
        CatalogAssurance::UncertifiedQuery
    );
    assert_eq!(
        crate::hex::hex_bytes(&route.catalog_digest),
        expected_digest
    );
    assert_eq!(route.matched_range.subnet_principal, SUBNET_A);
}

#[test]
fn assurance_minimums_are_ordered_from_query_to_certificate() {
    let cases = [
        (
            CatalogAssurance::UncertifiedQuery,
            CatalogAssurance::UncertifiedQuery,
            true,
        ),
        (
            CatalogAssurance::MultiEndpointAgreement,
            CatalogAssurance::UncertifiedQuery,
            true,
        ),
        (
            CatalogAssurance::MultiEndpointAgreement,
            CatalogAssurance::MultiEndpointAgreement,
            true,
        ),
        (
            CatalogAssurance::UncertifiedQuery,
            CatalogAssurance::MultiEndpointAgreement,
            false,
        ),
        (
            CatalogAssurance::MultiEndpointAgreement,
            CatalogAssurance::Certified,
            false,
        ),
        (
            CatalogAssurance::Certified,
            CatalogAssurance::MultiEndpointAgreement,
            true,
        ),
    ];

    for (actual, minimum, expected) in cases {
        assert_eq!(actual.satisfies(minimum), expected);
    }
}

#[test]
fn authority_validation_rejects_future_and_unclean_source_evidence() {
    let mut future = fixture_catalog();
    future.provenance.fetched_at = "2099-01-01T00:00:00Z".to_string();
    future.canonicalize_and_seal().expect("seal future fixture");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(future, &validation_context()),
        Err(CatalogError::FutureTimestamp { .. })
    ));

    let mut unclean_endpoint = fixture_catalog();
    unclean_endpoint.provenance.source_endpoints =
        vec!["https://reader:secret@example.com/?query=1".to_string()];
    unclean_endpoint
        .canonicalize_and_seal()
        .expect("seal endpoint fixture");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(unclean_endpoint, &validation_context()),
        Err(CatalogError::InvalidSourceEndpoint { .. })
    ));
}

#[test]
fn certified_and_incomplete_agreement_claims_fail_closed() {
    let mut raw = fixture_catalog();
    raw.provenance.assurance = CatalogAssurance::Certified;
    raw.provenance.certified_registry = Some(CertifiedRegistryCatalogEvidence {
        archive_manifest_schema_version: 1,
        delta_report_schema_version: 3,
        replay_provenance_schema_version: 1,
        root_key_digest: "00".repeat(32),
        evidence_chain_digest: "11".repeat(32),
        complete_state_digest: "22".repeat(32),
        minimum_certificate_time_nanos: 1_780_531_200_000_000_000,
        maximum_certificate_time_nanos: 1_780_531_200_000_000_000,
    });
    raw.canonicalize_and_seal()
        .expect("seal claimed certificate");

    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(raw, &validation_context()),
        Err(CatalogError::UnsupportedAssurance { assurance }) if assurance == "certified"
    ));

    let mut agreement = fixture_catalog();
    agreement.provenance.assurance = CatalogAssurance::MultiEndpointAgreement;
    agreement.provenance.source_endpoints = vec![
        "https://ic0.app".to_string(),
        "https://icp-api.io".to_string(),
    ];
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(agreement, &validation_context()),
        Err(CatalogError::InvalidAgreementDigest { .. })
    ));
}

#[test]
fn raw_registry_kind_charging_and_policy_relations_fail_closed() {
    let mut kind = fixture_catalog();
    kind.subnets[0].registry_subnet_type = 2;
    assert!(matches!(
        kind.validate(),
        Err(CatalogError::SubnetKindMismatch { .. })
    ));

    let mut charging = fixture_catalog();
    charging.subnets[0].charges_apply_by_default = false;
    assert!(matches!(
        charging.validate(),
        Err(CatalogError::ChargingPolicyMismatch { .. })
    ));

    let mut annotation = fixture_catalog();
    annotation.subnets[0].subnet_label = "edited".to_string();
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(annotation, &validation_context()),
        Err(CatalogError::ClassificationMismatch { .. })
    ));
}

#[test]
fn authority_validation_recomputes_catalog_digest_and_mainnet_identity() {
    let mut digest = fixture_catalog();
    digest.catalog_digest = "ff".repeat(32);
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(digest, &validation_context()),
        Err(CatalogError::CatalogDigestMismatch { .. })
    ));

    let mut identity = fixture_catalog();
    identity.provenance.registry_canister_id = "aaaaa-aa".to_string();
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(identity, &validation_context()),
        Err(CatalogError::RegistryCanisterMismatch { .. })
    ));
}

#[test]
fn authority_validation_requires_canonical_time_and_current_policy_identity() {
    let mut timestamp = fixture_catalog();
    timestamp.provenance.fetched_at = "2026-6-4T0:0:0Z".to_string();
    timestamp
        .canonicalize_and_seal()
        .expect("seal noncanonical timestamp");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(timestamp, &validation_context()),
        Err(CatalogError::InvalidTimestamp { .. })
    ));

    let mut classification = fixture_catalog();
    classification.provenance.classification_policy_digest = "ff".repeat(32);
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(classification, &validation_context()),
        Err(CatalogError::ClassificationPolicyDigestMismatch { .. })
    ));

    let mut resolver = fixture_catalog();
    resolver.provenance.resolver_schema_version += 1;
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(resolver, &validation_context()),
        Err(CatalogError::ResolverPolicyMismatch { .. })
    ));
}

#[test]
fn authority_validation_requires_call_counts_and_recomputes_agreement_digest() {
    let mut zero_calls = fixture_catalog();
    zero_calls.provenance.registry_query_call_count = 0;
    zero_calls
        .canonicalize_and_seal()
        .expect("seal zero-call fixture");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(zero_calls, &validation_context()),
        Err(CatalogError::InvalidProvenance {
            field: "provenance.registry_query_call_count",
            ..
        })
    ));

    let mut agreement = fixture_catalog();
    agreement
        .promote_to_multi_endpoint_agreement(
            vec![
                "https://alpha.example".to_string(),
                "https://beta.example".to_string(),
            ],
            10,
        )
        .expect("promote fixture agreement");
    ValidatedSubnetCatalog::try_from_raw(agreement.clone(), &validation_context())
        .expect("valid agreement");

    agreement.provenance.agreement_digest = Some("ff".repeat(32));
    agreement
        .canonicalize_and_seal()
        .expect("reseal tampered agreement metadata");
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(agreement, &validation_context()),
        Err(CatalogError::AgreementDigestMismatch { .. })
    ));
}
