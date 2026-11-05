use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row(&'static str);

#[test]
fn resolves_normalized_text_exactly_and_principals_by_prefix() {
    let text_rows = [Row("dc1")];
    assert_eq!(
        resolve_nns_inventory_row(
            &text_rows,
            |row| row.0,
            " DC1 ",
            NnsInventoryInputKind::Text,
            "id"
        ),
        Ok((0, "id".to_string()))
    );

    assert_eq!(
        resolve_nns_inventory_row(
            &text_rows,
            |row| row.0,
            " DC ",
            NnsInventoryInputKind::Text,
            "id"
        ),
        Ok((0, "id_prefix".to_string()))
    );

    let principal_rows = [Row("ryjl3-tyaaa-aaaaa-aaaba-cai")];
    assert_eq!(
        resolve_nns_inventory_row(
            &principal_rows,
            |row| row.0,
            "ryjl",
            NnsInventoryInputKind::Principal,
            "principal",
        ),
        Ok((0, "principal_prefix".to_string(),))
    );
}

#[test]
fn preserves_not_found_input_and_ambiguous_matches() {
    let rows = [Row("dc1"), Row("dc2")];
    assert_eq!(
        resolve_nns_inventory_row(&rows, |row| row.0, " ", NnsInventoryInputKind::Text, "id"),
        Err(NnsInventoryResolveError::NotFound {
            input: " ".to_string(),
        })
    );
    assert_eq!(
        resolve_nns_inventory_row(&rows, |row| row.0, "DC", NnsInventoryInputKind::Text, "id"),
        Err(NnsInventoryResolveError::Ambiguous {
            prefix: "dc".to_string(),
            matches: vec!["dc1".to_string(), "dc2".to_string()],
        })
    );
}

struct SharedInventorySource(std::sync::atomic::AtomicUsize);
impl crate::nns::NnsRegistryInventorySource for SharedInventorySource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a crate::report::CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::NnsRegistryInventorySnapshot, crate::nns::NnsInventoryError>,
    > {
        Box::pin(async move {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let providers = [crate::nns::node_provider::NnsNodeProviderRow {
                node_provider_principal: "provider".to_string(),
                name: None,
                node_count: Some(2),
                reward_account_hex: None,
            }];
            let mut snapshot = crate::tests::inventory_for_provider_rows(request, &providers);
            snapshot
                .node_operators
                .push(crate::nns::node_operator::NnsNodeOperatorRow {
                    node_operator_principal: "fixture-operator".to_string(),
                    node_provider_principal: "provider".to_string(),
                    node_count: Some(2),
                    node_allowance: 3,
                    data_center_id: "fixture".to_string(),
                });
            snapshot
                .data_centers
                .push(crate::nns::data_center::NnsDataCenterRow {
                    data_center_id: "fixture".to_string(),
                    region: "region".to_string(),
                    owner: "owner".to_string(),
                    latitude: None,
                    longitude: None,
                    node_count: 2,
                    node_operator_count: 1,
                    node_provider_count: 1,
                });
            Ok(snapshot)
        })
    }
}
impl crate::nns::node_provider::NnsNodeProviderSource for SharedInventorySource {
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        _request: &'a crate::report::CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::node_provider::NnsNodeProviderMetadata, crate::nns::NnsInventoryError>,
    > {
        Box::pin(async {
            Err(crate::nns::NnsInventoryError::InvalidSourceData {
                collection: crate::nns::NnsInventoryCollection::NodeProvider,
                reason: "Governance unavailable".to_string(),
            })
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn registry_views_share_one_pin_and_survive_governance_failure() {
    use crate::nns::{NnsInventoryCacheRequest, NnsInventoryListRequest, node::NnsNodeListRequest};
    let root = crate::tests::temp_dir("ic-query-shared-registry-inventory");
    let cache = NnsInventoryCacheRequest::new(&root, "ic");
    let request = NnsInventoryListRequest::new(cache.clone(), "https://icp-api.io", 1_780_531_200);
    let source = SharedInventorySource(std::sync::atomic::AtomicUsize::new(0));
    let mut progress = Vec::new();
    let nodes = crate::nns::node::build_nns_node_list_report_with_source_and_progress(
        &NnsNodeListRequest::new(
            cache.clone(),
            &request.source_endpoint,
            request.now_unix_secs,
        ),
        &source,
        &mut |event| progress.push(event),
    )
    .await
    .unwrap();
    let operators =
        crate::nns::node_operator::build_nns_node_operator_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |event| progress.push(event),
        )
        .await
        .unwrap();
    let centres =
        crate::nns::data_center::build_nns_data_center_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |event| progress.push(event),
        )
        .await
        .unwrap();
    assert_eq!(source.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(progress.len(), 1);
    assert_eq!(nodes.nodes.len(), 2);
    assert_eq!(operators.node_operators.len(), 1);
    assert_eq!(centres.data_centers.len(), 1);
    assert_eq!(nodes.provenance, operators.provenance);
    assert_eq!(nodes.provenance, centres.provenance);
    let path = super::nns_registry_inventory_cache_path(&root, "ic");
    let before = std::fs::read(&path).unwrap();
    assert!(matches!(
        crate::nns::node_provider::build_nns_node_provider_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |_| {}
        )
        .await,
        Err(crate::nns::NnsInventoryError::InvalidSourceData {
            collection: crate::nns::NnsInventoryCollection::NodeProvider,
            ..
        })
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!crate::nns::node_provider::nns_node_provider_cache_path(&root, "ic").exists());
    crate::nns::data_center::build_nns_data_center_list_report_with_source_and_progress(
        &request,
        &source,
        &mut |_| {},
    )
    .await
    .unwrap();
    assert_eq!(source.0.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn shared_snapshot_rejects_unknown_and_duplicate_cache_fields() {
    let source = SharedInventorySource(std::sync::atomic::AtomicUsize::new(0));
    let request = crate::report::CollectionSourceRequest::from_unix_secs(
        "ic",
        "https://icp-api.io",
        1_780_531_200,
        "fixture",
    );
    let snapshot = source.fetch_registry_inventory(&request).await.unwrap();
    let original = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<NnsRegistryInventorySnapshot>(&original).unwrap(),
        snapshot
    );
    for extra in [
        "\"unexpected\":true,",
        "\"schema_version\":1,",
        "\"registry_version\":42,",
    ] {
        let bad = format!("{{{extra}{}", &original[1..]);
        assert!(serde_json::from_str::<NnsRegistryInventorySnapshot>(&bad).is_err());
    }
    let mut nested = serde_json::to_value(&snapshot).unwrap();
    nested["nodes"][0]["unexpected"] = true.into();
    assert!(serde_json::from_value::<NnsRegistryInventorySnapshot>(nested).is_err());
}
