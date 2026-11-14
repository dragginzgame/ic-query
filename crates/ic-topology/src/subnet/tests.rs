use ic_platform_types::Principal;

use super::{MAX_NODES, SubnetError, project};

#[test]
fn stable_inventory_preserves_source_identity_and_qualified_claims() {
    let subnet = Principal::from_slice(&[10]);
    let nodes = [Principal::from_slice(&[2]), Principal::from_slice(&[1])];
    let report = project(subnet, subnet, nodes, 9_007_199_254_740_993).unwrap();
    assert_eq!(report.observed_node_ids, [nodes[1], nodes[0]]);
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["subnet_id"], subnet.to_text());
    assert_eq!(json["observed_node_ids"][0], nodes[1].to_text());
    assert_eq!(json["evidence"]["state_certificate_verified"], true);
    assert_eq!(json["evidence"]["membership_complete"], false);
    assert_eq!(
        json["evidence"]["observed_at_unix_seconds"].as_u64(),
        Some(9_007_199_254_740_993)
    );
    assert!(json["evidence"]["registry_version"].is_null());
    assert_eq!(
        json["evidence"]["source_path"],
        format!("/api/v3/subnet/{subnet}/read_state")
    );
}

#[test]
fn wrong_subnet_empty_and_duplicate_observations_are_refused() {
    let subnet = Principal::from_slice(&[10]);
    let node = Principal::from_slice(&[1]);
    assert!(matches!(
        project(subnet, node, [node], 0),
        Err(SubnetError::IdentityMismatch { .. })
    ));
    assert!(matches!(
        project(subnet, subnet, [], 0),
        Err(SubnetError::MissingNodes)
    ));
    assert!(matches!(
        project(subnet, subnet, [node, node], 0),
        Err(SubnetError::DuplicateNode)
    ));
}

#[test]
fn exact_node_limit_succeeds_and_excess_is_not_truncated() {
    let subnet = Principal::from_slice(&[10]);
    let nodes = (0..MAX_NODES)
        .map(|index| Principal::from_slice(&index.to_be_bytes()))
        .collect::<Vec<_>>();
    assert_eq!(
        project(subnet, subnet, nodes, 0)
            .unwrap()
            .observed_node_ids
            .len(),
        MAX_NODES
    );
    let mut consumed = 0;
    let unbounded = std::iter::from_fn(|| {
        consumed += 1;
        Some(Principal::from_slice(&[1]))
    });
    assert!(matches!(
        project(subnet, subnet, unbounded, 0),
        Err(SubnetError::NodeLimit)
    ));
    assert_eq!(consumed, MAX_NODES + 1);
}
