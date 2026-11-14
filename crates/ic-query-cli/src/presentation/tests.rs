use ic_platform_types::Principal;
use ic_topology::placement::{PlacementEvidence, PlacementReport};
use ic_topology::subnet::list::{SubnetListEvidence, SubnetListReport};
use ic_topology::subnet::{SubnetEvidence, SubnetReport};

use super::{placement, subnet, subnet_list};

#[test]
fn subnet_list_text_labels_record_version_and_preserves_evidence() {
    let id = Principal::from_slice(&[1]);
    let report = SubnetListReport {
        schema: 1,
        subnet_ids: vec![id],
        evidence: SubnetListEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_list",
            observed_at_unix_seconds: 123,
            record_version: 456,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: false,
            result_certified: false,
            selected_record_complete: true,
        },
    };
    let text = subnet_list(&report);
    assert!(text.contains("Record mutation version: 456"));
    assert!(text.contains("Record mutation timestamp: 18446744073709551615 Unix nanoseconds"));
    assert!(text.contains("Query signature verified: false"));
    assert!(text.ends_with(&format!(
        "Selected record complete: true\n\nRegistered subnets (1):\n{id}"
    )));
}

#[test]
fn placement_text_uses_evidence_instead_of_hardcoded_verification_claims() {
    let report = PlacementReport {
        schema: 1,
        canister_id: Principal::from_slice(&[1]),
        subnet_id: Principal::from_slice(&[2]),
        evidence: PlacementEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "lookup",
            observed_at_unix_seconds: 123,
            query_signature_verified: false,
            result_certified: false,
            registry_version: Some(456),
            selected_lookup_complete: true,
        },
    };
    let text = placement(&report);
    assert!(text.contains("Query signature verified: false"));
    assert!(text.contains("Registry version: 456\n\nCanister:"));
    assert!(text.contains(&format!("Subnet: {}", report.subnet_id)));
}

#[test]
fn subnet_text_preserves_coverage_and_separates_metadata_from_inventory() {
    let node = Principal::from_slice(&[1]);
    let report = SubnetReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[2]),
        observed_node_ids: vec![node],
        evidence: SubnetEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            observed_at_unix_seconds: 123,
            state_certificate_verified: true,
            membership_complete: false,
            registry_version: None,
        },
    };
    let text = subnet(&report);
    assert!(text.contains("Complete Registry membership established: false"));
    assert!(text.contains("Registry version: unavailable\n\nSubnet:"));
    assert!(text.ends_with(&format!("Observed node identities (1):\n{node}")));
}
