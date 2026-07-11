use super::fixtures::{SUBNET_A, SUBNET_B, fixture_catalog, sorted_principals};
use crate::subnet_catalog::{CatalogError, RoutingRange};

const THREE_ITEM_ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

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
fn validation_rejects_overlapping_routing_ranges() {
    let ids = sorted_principals([
        "ryjl3-tyaaa-aaaaa-aaaba-cai",
        "rrkah-fqaaa-aaaaa-aaaaq-cai",
        "r7inp-6aaaa-aaaaa-aaabq-cai",
    ]);
    let mut catalog = fixture_catalog();
    catalog.routing_ranges = vec![
        RoutingRange {
            start_canister_id: ids[0].clone(),
            end_canister_id: ids[2].clone(),
            subnet_principal: SUBNET_A.to_string(),
        },
        RoutingRange {
            start_canister_id: ids[1].clone(),
            end_canister_id: ids[2].clone(),
            subnet_principal: SUBNET_B.to_string(),
        },
    ];

    assert!(matches!(
        catalog.validate(),
        Err(CatalogError::OverlappingRoutingRanges { .. })
    ));
}

#[test]
fn routing_range_validation_is_independent_of_input_order() {
    let ids = sorted_principals([
        "ryjl3-tyaaa-aaaaa-aaaba-cai",
        "rrkah-fqaaa-aaaaa-aaaaq-cai",
        "r7inp-6aaaa-aaaaa-aaabq-cai",
    ]);
    let disjoint = ids
        .iter()
        .enumerate()
        .map(|(index, id)| RoutingRange {
            start_canister_id: id.clone(),
            end_canister_id: id.clone(),
            subnet_principal: if index == 1 { SUBNET_B } else { SUBNET_A }.to_string(),
        })
        .collect::<Vec<_>>();
    let overlapping = [
        RoutingRange {
            start_canister_id: ids[0].clone(),
            end_canister_id: ids[2].clone(),
            subnet_principal: SUBNET_A.to_string(),
        },
        RoutingRange {
            start_canister_id: ids[1].clone(),
            end_canister_id: ids[1].clone(),
            subnet_principal: SUBNET_B.to_string(),
        },
        disjoint[2].clone(),
    ];

    for order in THREE_ITEM_ORDERS {
        let mut valid = fixture_catalog();
        valid.routing_ranges = order.map(|index| disjoint[index].clone()).to_vec();
        valid
            .validate()
            .expect("disjoint ranges are valid in every order");

        let mut invalid = fixture_catalog();
        invalid.routing_ranges = order.map(|index| overlapping[index].clone()).to_vec();
        assert!(matches!(
            invalid.validate(),
            Err(CatalogError::OverlappingRoutingRanges { .. })
        ));
    }
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
