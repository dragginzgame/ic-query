use super::fixtures::*;
use super::*;

#[test]
fn topology_regions_report_summarizes_data_center_regions() {
    let report = topology_regions_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        data_center_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.regions.len(), 1);
    assert!(
        serde_json::to_value(&report)
            .unwrap()
            .get("region_count")
            .is_none()
    );
    assert_eq!(report.data_center_count, 1);
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.node_provider_count, 1);
    assert_eq!(report.node_count, 3);
    assert_eq!(report.regions[0].region, "eu-west");
    assert_eq!(report.regions[0].data_center_count, 1);
}
