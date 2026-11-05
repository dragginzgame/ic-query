use super::fixtures::*;
use super::*;

#[tokio::test(flavor = "current_thread")]
async fn topology_versions_report_preserves_component_registry_provenance() {
    let report = build_nns_topology_versions_report_with_source(
        &topology_read_request_fixture(),
        &RecordingTopologySource::default(),
    )
    .await
    .expect("topology versions report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.registry_versions.len(), 5);
    assert_eq!(report.registry_versions[0].source, "subnet_catalog");
    assert_eq!(report.registry_versions[1].source, "nodes");
    let summary = build_nns_topology_summary_report_with_source(
        &topology_read_request_fixture(),
        &RecordingTopologySource::default(),
    )
    .await
    .expect("topology summary report");
    assert_eq!(report.registry_versions, summary.registry_versions);
    assert_eq!(report.registry_versions[0].stale, Some(false));
    assert_eq!(report.registry_versions[0].source_endpoint, "-");
    for (row, version) in report.registry_versions.iter().zip([42, 43, 43, 43, 43]) {
        assert_eq!(row.registry_version, version);
        assert_ne!(row.fetched_at, "");
    }
}
