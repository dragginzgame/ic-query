use super::{fixtures::*, *};

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_counts_component_reports() {
    let source = RecordingTopologySource::default();
    let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let report = refresh_nns_topology_report_with_source(&request, &source)
        .await
        .expect("topology refresh");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.component_count, 5);
    assert_eq!(report.wrote_cache_count, 5);
    assert_eq!(report.replaced_existing_cache_count, 1);
    assert_eq!(report.components[0].source, "subnet_catalog");
    assert_eq!(report.components[0].item_count, 2);
    assert_eq!(report.components[1].source, "nodes");
    assert_eq!(report.components[1].item_count, 3);
    assert_eq!(report.components[2].source, "node_providers");
    assert_eq!(report.components[2].item_count, 1);
    assert_eq!(report.components[3].source, "node_operators");
    assert_eq!(report.components[3].item_count, 2);
    assert_eq!(report.components[4].source, "data_centers");
    assert_eq!(report.components[4].item_count, 1);
    assert_eq!(
        *source.calls.lock().unwrap(),
        [
            "subnet_catalog",
            "nodes",
            "node_providers",
            "node_operators",
            "data_centers"
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_text_renders_component_table() {
    let request =
        NnsTopologyRefreshRequest::new(topology_read_request_fixture()).with_dry_run(true);
    let report =
        refresh_nns_topology_report_with_source(&request, &RecordingTopologySource::default())
            .await
            .expect("dry-run topology refresh");

    let text = nns_topology_refresh_report_text(&report);

    assert!(text.contains("topology_refresh: ic components 5 wrote 0 replaced 1 dry_run yes"));
    assert!(text.contains("subnet_catalog"));
    assert!(text.contains("node_operators"));
    assert!(text.contains("data_centers"));
}

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_stops_at_the_first_source_failure() {
    let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let components = [
        "subnet_catalog",
        "nodes",
        "node_providers",
        "node_operators",
        "data_centers",
    ];
    for (index, component) in components.iter().enumerate() {
        let source = RecordingTopologySource {
            fail_at: Some(component),
            ..Default::default()
        };
        let error = refresh_nns_topology_report_with_source(&request, &source)
            .await
            .expect_err("source failure");
        assert!(
            matches!(error, NnsTopologyHostError::Node(crate::nns::node::NnsNodeHostError::InvalidSourceData { reason }) if reason == *component)
        );
        assert_eq!(*source.calls.lock().unwrap(), components[..=index]);
    }
    let mut request = request;
    request.read.cache.network = "local".to_string();
    let source = RecordingTopologySource::default();
    assert!(
        matches!(refresh_nns_topology_report_with_source(&request, &source).await,
        Err(NnsTopologyHostError::UnsupportedNetwork { network }) if network == "local")
    );
    assert!(source.calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_rejects_mismatched_component_collections() {
    let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let components = [
        "subnet_catalog",
        "nodes",
        "node_providers",
        "node_operators",
        "data_centers",
    ];
    for (index, component) in components.iter().enumerate().skip(1) {
        let source = RecordingTopologySource {
            wrong_refresh_collection_at: Some(component),
            ..Default::default()
        };
        let error = refresh_nns_topology_report_with_source(&request, &source)
            .await
            .expect_err("collection mismatch");
        let rejected_collection = match error {
            NnsTopologyHostError::Node(crate::nns::node::NnsNodeHostError::InvalidSourceData {
                ..
            }) => "nodes",
            NnsTopologyHostError::NodeProvider(
                crate::nns::node_provider::NnsNodeProviderHostError::InvalidSourceData { .. },
            ) => "node_providers",
            NnsTopologyHostError::NodeOperator(
                crate::nns::node_operator::NnsNodeOperatorHostError::InvalidSourceData { .. },
            ) => "node_operators",
            NnsTopologyHostError::DataCenter(
                crate::nns::data_center::NnsDataCenterHostError::InvalidSourceData { .. },
            ) => "data_centers",
            other => panic!("unexpected refresh failure: {other}"),
        };
        assert_eq!(rejected_collection, *component);
        assert_eq!(*source.calls.lock().unwrap(), components[..=index]);
    }
}
