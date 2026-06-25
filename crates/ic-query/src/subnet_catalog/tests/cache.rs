use super::*;

#[test]
fn catalog_path_lives_outside_deployment_state() {
    let root = PathBuf::from("/tmp/ic-query-project");

    let path = subnet_catalog_path(&root, MAINNET_NETWORK);

    assert_eq!(
        path,
        PathBuf::from("/tmp/ic-query-project/.icq/subnet-catalog/ic/catalog.json")
    );
    assert!(!path.display().to_string().contains("/deployments/"));
    assert!(!path.display().to_string().contains("/fleets/"));
}

#[test]
fn load_cached_catalog_rejects_non_mainnet_network() {
    let root = temp_dir("ic-query-subnet-network");
    let request = SubnetCatalogCacheRequest {
        icp_root: root.clone(),
        network: "local".to_string(),
    };

    let err = load_cached_subnet_catalog(&request).expect_err("local rejected");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        err,
        SubnetCatalogHostError::UnsupportedNetwork { .. }
    ));
}

#[test]
fn missing_catalog_error_explains_cached_only_slice() {
    let root = temp_dir("ic-query-subnet-missing");
    let request = SubnetCatalogCacheRequest {
        icp_root: root.clone(),
        network: MAINNET_NETWORK.to_string(),
    };

    let err = load_cached_subnet_catalog(&request).expect_err("cache missing");
    let message = err.to_string();

    let _ = fs::remove_dir_all(root);
    assert!(message.contains("Run `icq nns subnet refresh`"));
    assert!(message.contains("public Internet Computer mainnet catalog"));
    assert!(message.contains("icq nns subnet refresh"));
}
