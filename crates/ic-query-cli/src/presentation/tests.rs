use ic_platform_types::Principal;
use ic_topology::placement::{PlacementEvidence, PlacementReport};
use ic_topology::subnet::{SubnetEvidence, SubnetReport};

use super::{placement, subnet};

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
