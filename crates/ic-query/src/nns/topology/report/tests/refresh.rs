use super::{fixtures::*, *};

#[test]
fn topology_refresh_counts_component_reports() {
    let report = topology_refresh_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        false,
        NnsTopologyRefreshComponentReports {
            subnet: subnet_refresh_report_fixture(),
            node: node_refresh_report_fixture(),
            node_provider: node_provider_refresh_report_fixture(),
            node_operator: node_operator_refresh_report_fixture(),
            data_center: data_center_refresh_report_fixture(),
        },
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.component_count, 5);
    assert_eq!(report.wrote_cache_count, 5);
    assert_eq!(report.replaced_existing_cache_count, 1);
    assert_eq!(report.components[0].source, "subnet_catalog");
    assert_eq!(report.components[0].item_count, 2);
    assert_eq!(report.components[1].source, "nodes");
    assert_eq!(report.components[1].item_count, 3);
}

#[test]
fn topology_refresh_text_renders_component_table() {
    let report = topology_refresh_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        true,
        NnsTopologyRefreshComponentReports {
            subnet: dry_run_subnet_refresh_report_fixture(),
            node: dry_run_node_refresh_report_fixture(),
            node_provider: dry_run_node_provider_refresh_report_fixture(),
            node_operator: dry_run_node_operator_refresh_report_fixture(),
            data_center: dry_run_data_center_refresh_report_fixture(),
        },
    );

    let text = nns_topology_refresh_report_text(&report);

    assert!(text.contains("topology_refresh: ic components 5 wrote 0 replaced 1 dry_run yes"));
    assert!(text.contains("subnet_catalog"));
    assert!(text.contains("node_operators"));
    assert!(text.contains("data_centers"));
}
