use super::fixtures::{SUBNET_A, SUBNET_B, fixture_catalog, sorted_principals};
use crate::subnet_catalog::{
    CatalogError, ClassificationSource, GeographicScope, ResolveAs, ResolvedSubnetSubject,
    RoutingRange, SubnetInfo, SubnetKind, SubnetSpecialization,
};

#[test]
fn resolver_maps_canister_by_inclusive_principal_byte_range() {
    let ids = sorted_principals([
        "ryjl3-tyaaa-aaaaa-aaaba-cai",
        "rrkah-fqaaa-aaaaa-aaaaq-cai",
        "r7inp-6aaaa-aaaaa-aaabq-cai",
        "t63gs-up777-77776-aaaba-cai",
        "uxrrr-q7777-77774-qaaaq-cai",
    ]);
    let mut catalog = fixture_catalog();
    catalog.routing_ranges = vec![RoutingRange {
        start_canister_id: ids[1].clone(),
        end_canister_id: ids[3].clone(),
        subnet_principal: SUBNET_A.to_string(),
    }];
    catalog.validate().expect("valid range");

    let start = catalog
        .resolve_principal(&ids[1], None)
        .expect("start boundary resolves");
    let inside = catalog
        .resolve_principal(&ids[2], None)
        .expect("inside resolves");
    let end = catalog
        .resolve_principal(&ids[3], None)
        .expect("end boundary resolves");

    assert_eq!(start.subnet.subnet_principal, SUBNET_A);
    assert_eq!(inside.subnet.subnet_principal, SUBNET_A);
    assert_eq!(end.subnet.subnet_principal, SUBNET_A);
    assert!(matches!(
        catalog.resolve_principal(&ids[0], None),
        Err(CatalogError::RouteNotFound { .. })
    ));
    assert!(matches!(
        catalog.resolve_principal(&ids[4], None),
        Err(CatalogError::RouteNotFound { .. })
    ));
}

#[test]
fn known_subnet_principal_wins_over_canister_range_interpretation() {
    let mut catalog = fixture_catalog();
    catalog.routing_ranges = vec![RoutingRange {
        start_canister_id: SUBNET_A.to_string(),
        end_canister_id: SUBNET_A.to_string(),
        subnet_principal: SUBNET_B.to_string(),
    }];
    catalog.validate().expect("valid synthetic overlap");

    let resolved = catalog
        .resolve_principal(SUBNET_A, None)
        .expect("known subnet wins");

    assert_eq!(resolved.resolved_as, ResolvedSubnetSubject::Subnet);
    assert_eq!(resolved.subnet.subnet_principal, SUBNET_A);
}

#[test]
fn forced_interpretations_are_explicit() {
    let catalog = fixture_catalog();

    let subnet = catalog
        .resolve_principal(SUBNET_A, Some(ResolveAs::Subnet))
        .expect("forced subnet");
    assert_eq!(subnet.resolved_as, ResolvedSubnetSubject::Subnet);

    let err = catalog
        .resolve_principal(SUBNET_A, Some(ResolveAs::Canister))
        .expect_err("subnet principal is not covered as a canister");
    assert!(matches!(err, CatalogError::RouteNotFound { .. }));
}

#[test]
fn resolver_accepts_unique_subnet_principal_prefix() {
    let catalog = fixture_catalog();

    let resolved = catalog
        .resolve_principal_or_prefix("rwl", None)
        .expect("subnet prefix resolves");

    assert_eq!(resolved.resolved_as, ResolvedSubnetSubject::Subnet);
    assert_eq!(resolved.resolved_from, "subnet_principal_prefix");
    assert_eq!(resolved.input_principal, "rwl");
    assert_eq!(resolved.subnet.subnet_principal, SUBNET_A);
}

#[test]
fn resolver_rejects_canister_boundary_prefix() {
    let catalog = fixture_catalog();

    let err = catalog
        .resolve_principal_or_prefix("ryj", None)
        .expect_err("partial canister principals are not accepted");

    assert!(matches!(
        err,
        CatalogError::PrincipalPrefixNotFound { prefix } if prefix == "ryj"
    ));
}

#[test]
fn resolver_rejects_ambiguous_principal_prefix() {
    let mut catalog = fixture_catalog();
    catalog.subnets.push(SubnetInfo {
        subnet_principal: "r7inp-6aaaa-aaaaa-aaabq-cai".to_string(),
        subnet_kind: SubnetKind::Application,
        subnet_kind_source: ClassificationSource::Registry,
        subnet_specialization: SubnetSpecialization::None,
        subnet_specialization_source: ClassificationSource::Computed,
        geographic_scope: GeographicScope::Global,
        geographic_scope_source: ClassificationSource::Computed,
        subnet_label: "application".to_string(),
        subnet_label_source: ClassificationSource::Computed,
        node_count: Some(13),
        charges_apply_by_default: true,
    });
    catalog.validate().expect("valid ambiguous fixture");

    let err = catalog
        .resolve_principal_or_prefix("r", None)
        .expect_err("ambiguous prefix fails");

    assert!(matches!(
        err,
        CatalogError::AmbiguousPrincipalPrefix { prefix, matches }
            if prefix == "r" && matches.len() > 1
    ));
}

#[test]
fn resolver_returns_typed_error_for_unknown_routing_subnet() {
    let mut catalog = fixture_catalog();
    catalog.routing_ranges[0].subnet_principal = "uxrrr-q7777-77774-qaaaq-cai".to_string();

    let err = catalog
        .resolve_principal("ryjl3-tyaaa-aaaaa-aaaba-cai", None)
        .expect_err("missing routing subnet fails without panic");

    assert!(matches!(
        err,
        CatalogError::UnknownRoutingSubnet {
            subnet_principal
        } if subnet_principal == "uxrrr-q7777-77774-qaaaq-cai"
    ));
}
