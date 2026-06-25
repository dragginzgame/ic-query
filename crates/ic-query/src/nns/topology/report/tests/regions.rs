use super::{fixtures::*, *};

#[test]
fn topology_regions_report_summarizes_data_center_regions() {
    let report = topology_regions_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        data_center_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.region_count, 1);
    assert_eq!(report.data_center_count, 1);
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.node_provider_count, 1);
    assert_eq!(report.node_count, 3);
    assert_eq!(report.regions[0].region, "eu-west");
    assert_eq!(report.regions[0].data_center_count, 1);
}

#[test]
fn topology_regions_text_renders_region_table() {
    let report = topology_regions_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        data_center_report_fixture(),
    );

    let text = nns_topology_regions_report_text(&report);

    assert!(text.contains("REGION"));
    assert!(text.contains("DATA_CENTERS"));
    assert!(text.contains("NODE_OPERATORS"));
    assert!(text.contains("eu-west"));
}
