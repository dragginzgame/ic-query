use super::{fixtures::*, *};

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_publishes_one_registry_snapshot_and_independent_metadata() {
    let source = RecordingTopologySource::default();
    let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let report = refresh_nns_topology_report_with_source(&request, &source)
        .await
        .unwrap();
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.component_count, 3);
    assert_eq!(report.wrote_cache_count, 3);
    assert_eq!(
        report
            .components
            .iter()
            .map(|r| r.source.as_str())
            .collect::<Vec<_>>(),
        ["subnet_catalog", "inventory", "node_provider_metadata"]
    );
    assert_eq!(report.components[1].item_count, 6);
    assert!(report.components[1].registry_version.is_some());
    assert_eq!(report.components[2].registry_version, None);
    assert_eq!(
        *source.calls.lock().unwrap(),
        ["subnet_catalog", "inventory", "metadata"]
    );
    let snapshot = crate::nns::inventory::load_registry_inventory(&request.read.cache).unwrap();
    assert_eq!(snapshot.nodes.len(), 3);
    assert_eq!(snapshot.node_operators.len(), 2);
    assert_eq!(snapshot.data_centers.len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_stops_at_the_first_source_failure() {
    let components = ["subnet_catalog", "inventory", "metadata"];
    for (index, component) in components.iter().enumerate() {
        let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
        let source = RecordingTopologySource {
            fail_at: Some(component),
            ..Default::default()
        };
        let error = refresh_nns_topology_report_with_source(&request, &source)
            .await
            .unwrap_err();
        assert!(
            matches!(error, NnsTopologyHostError::Inventory(crate::nns::NnsInventoryError::InvalidSourceData { reason, .. }) if reason == *component)
        );
        assert_eq!(*source.calls.lock().unwrap(), components[..=index]);
    }
    let mut request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    request.read.cache.network = "local".to_string();
    let source = RecordingTopologySource::default();
    assert!(matches!(
        refresh_nns_topology_report_with_source(&request, &source).await,
        Err(NnsTopologyHostError::UnsupportedNetwork { .. })
    ));
    assert!(source.calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn topology_refresh_rejects_unadmitted_inventory_before_publication() {
    let request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let source = RecordingTopologySource {
        invalid_inventory: true,
        ..Default::default()
    };
    assert!(matches!(
        refresh_nns_topology_report_with_source(&request, &source).await,
        Err(NnsTopologyHostError::Inventory(
            crate::nns::NnsInventoryError::InvalidSourceData { .. }
        ))
    ));
    assert_eq!(
        *source.calls.lock().unwrap(),
        ["subnet_catalog", "inventory"]
    );
    assert!(
        !crate::nns::nns_registry_inventory_cache_path(&request.read.cache.cache_root, "ic")
            .exists()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn governance_failure_retains_previous_metadata_and_new_registry_snapshot() {
    let mut request = NnsTopologyRefreshRequest::new(topology_read_request_fixture());
    let initial =
        refresh_nns_topology_report_with_source(&request, &RecordingTopologySource::default())
            .await
            .unwrap();
    let metadata_path = &initial.components[2].cache_path;
    let original_metadata = std::fs::read(metadata_path).unwrap();
    let original_registry =
        crate::nns::inventory::load_registry_inventory(&request.read.cache).unwrap();
    request.read.now_unix_secs += 60;
    let source = RecordingTopologySource {
        fail_at: Some("metadata"),
        ..Default::default()
    };
    assert!(matches!(
        refresh_nns_topology_report_with_source(&request, &source).await,
        Err(NnsTopologyHostError::Inventory(
            crate::nns::NnsInventoryError::InvalidSourceData { .. }
        ))
    ));
    assert_eq!(std::fs::read(metadata_path).unwrap(), original_metadata);
    let current_registry =
        crate::nns::inventory::load_registry_inventory(&request.read.cache).unwrap();
    assert_ne!(
        current_registry.provenance.fetched_at,
        original_registry.provenance.fetched_at
    );
    assert_eq!(current_registry.nodes, original_registry.nodes);
    let providers =
        crate::nns::node_provider::build_nns_node_provider_list_report_with_source_and_progress(
            &request.read,
            &source,
            &mut |_| {},
        )
        .await
        .unwrap();
    assert_eq!(providers.provenance, current_registry.provenance);
    assert_eq!(
        providers.governance_source.fetched_at,
        original_registry.provenance.fetched_at
    );
}
