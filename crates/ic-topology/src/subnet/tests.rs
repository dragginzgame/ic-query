use ic_platform::agent::SubnetType;
use ic_platform_types::Principal;

use super::{MAX_NODES, MAX_SUBNET_TYPE_BYTES, SubnetError, project};

#[test]
fn stable_inventory_preserves_source_identity_and_qualified_claims() {
    let subnet = Principal::from_slice(&[10]);
    let nodes = [Principal::from_slice(&[2]), Principal::from_slice(&[1])];
    let report = project(subnet, subnet, nodes, 9_007_199_254_740_993, None).unwrap();
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
    assert!(json["observed_subnet_type"].is_null());
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
        project(subnet, node, [node], 0, None),
        Err(SubnetError::IdentityMismatch { .. })
    ));
    assert!(matches!(
        project(subnet, subnet, [], 0, None),
        Err(SubnetError::MissingNodes)
    ));
    assert!(matches!(
        project(subnet, subnet, [node, node], 0, None),
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
        project(subnet, subnet, nodes, 0, None)
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
        project(subnet, subnet, unbounded, 0, None),
        Err(SubnetError::NodeLimit)
    ));
    assert_eq!(consumed, MAX_NODES + 1);
}

#[test]
fn subnet_types_preserve_known_unknown_and_absent_values() {
    let id = Principal::from_slice(&[1]);
    for (source, expected) in [
        (None, None),
        (Some(SubnetType::System), Some("system")),
        (Some(SubnetType::Application), Some("application")),
        (
            Some(SubnetType::VerifiedApplication),
            Some("verified_application"),
        ),
        (Some(SubnetType::CloudEngine), Some("cloud_engine")),
        (Some(SubnetType::Unknown(String::new())), Some("")),
        (
            Some(SubnetType::Unknown("Future\n\u{1b}[31m型".into())),
            Some("Future\n\u{1b}[31m型"),
        ),
    ] {
        let report = project(id, id, [id], 0, source.as_ref()).unwrap();
        assert_eq!(report.observed_subnet_type.as_deref(), expected);
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(
            json["observed_subnet_type"],
            serde_json::to_value(expected).unwrap()
        );
    }
}

#[test]
fn subnet_type_budget_counts_utf8_bytes_and_never_truncates() {
    let id = Principal::from_slice(&[1]);
    let exact = "é".repeat(MAX_SUBNET_TYPE_BYTES / 2);
    let source = SubnetType::Unknown(exact.clone());
    let report = project(id, id, [id], 0, Some(&source)).unwrap();
    assert_eq!(report.observed_subnet_type, Some(exact.clone()));
    let excess = SubnetType::Unknown(format!("{exact}x"));
    assert!(matches!(
        project(id, id, [id], 0, Some(&excess)),
        Err(SubnetError::SubnetTypeLimit)
    ));
}
