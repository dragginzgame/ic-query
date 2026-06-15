use super::fixtures::{SUBNET_A, fixture_catalog, sorted_principals};
use crate::subnet_catalog::{CatalogError, RoutingRange};

#[test]
fn empty_subnets_and_empty_ranges_are_rejected() {
    let mut empty_subnets = fixture_catalog();
    empty_subnets.subnets.clear();
    assert!(matches!(
        empty_subnets.validate(),
        Err(CatalogError::EmptySubnets)
    ));

    let mut empty_ranges = fixture_catalog();
    empty_ranges.routing_ranges.clear();
    assert!(matches!(
        empty_ranges.validate(),
        Err(CatalogError::EmptyRoutingRanges)
    ));
}

#[test]
fn validation_rejects_unknown_routing_subnet_and_reversed_range() {
    let mut unknown = fixture_catalog();
    unknown.routing_ranges[0].subnet_principal = "uxrrr-q7777-77774-qaaaq-cai".to_string();
    assert!(matches!(
        unknown.validate(),
        Err(CatalogError::UnknownRoutingSubnet { .. })
    ));

    let ids = sorted_principals(["ryjl3-tyaaa-aaaaa-aaaba-cai", "rrkah-fqaaa-aaaaa-aaaaq-cai"]);
    let mut reversed = fixture_catalog();
    reversed.routing_ranges = vec![RoutingRange {
        start_canister_id: ids[1].clone(),
        end_canister_id: ids[0].clone(),
        subnet_principal: SUBNET_A.to_string(),
    }];
    assert!(matches!(
        reversed.validate(),
        Err(CatalogError::InvalidRoutingRange { .. })
    ));
}
