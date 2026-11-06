use super::fixtures::*;
use super::*;

#[test]
fn topology_check_report_flags_mixed_versions_and_unknown_joins() {
    let mut inventory = topology_inventory_fixture();
    inventory.registry.provenance.registry_version = 46;
    let report = topology_check_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        &inventory,
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, NnsTopologyAssessmentStatus::Attention);
    assert_eq!(report.registry_source_count, 5);
    assert_eq!(report.registry_version_min, Some(42));
    assert_eq!(report.registry_version_max, Some(46));
    assert!(!report.registry_versions_aligned);
    assert_eq!(report.stale_source_count, 0);
    assert_eq!(report.unknown_freshness_source_count, 4);
    assert_eq!(report.known_join_count, 8);
    assert_eq!(report.unknown_join_count, 5);
}
