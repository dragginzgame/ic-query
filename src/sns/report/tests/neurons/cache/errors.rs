use crate::sns::report::tests::{fixtures::*, *};
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn sns_neurons_cached_sort_requires_existing_complete_cache() {
    let root = temp_dir("ic-query-sns-neurons-missing-cache");
    let mut request = neurons_request("1");
    request.icp_root = Some(root.clone());
    request.sort = SnsNeuronsSort::Stake;

    let err = build_sns_neurons_report_with_source(&request, &NoLiveSnsNeuronsSource)
        .expect_err("missing cache is not auto-refreshed");

    assert!(matches!(
        err,
        SnsHostError::MissingNeuronsCacheForId { id: 1, .. }
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_refresh_max_pages_does_not_publish_incomplete_cache() {
    let root = temp_dir("ic-query-sns-neurons-incomplete-refresh");
    let request = sns_neurons_refresh_request(&root, Some(1));

    let err = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect_err("incomplete refresh");

    assert!(matches!(
        err,
        SnsHostError::IncompleteRefresh {
            pages_fetched: 1,
            rows_fetched: 2,
            ..
        }
    ));
    assert!(!sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A).exists());
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    assert!(attempt_path.is_file());

    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "failed");
    assert_eq!(attempt["pages_fetched"], 1);
    assert_eq!(attempt["rows_fetched"], 2);
    assert_eq!(attempt["last_cursor"], "02");
    assert!(
        attempt["last_error"]
            .as_str()
            .expect("last error")
            .contains("max pages reached before API exhaustion")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cached_sort_rejects_unsupported_cache_schema() {
    let root = temp_dir("ic-query-sns-neurons-unsupported-schema");
    let request = sns_neurons_refresh_request(&root, None);
    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");

    let cache_path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["schema_version"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    let mut cached_request = neurons_request("1");
    cached_request.icp_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    let err = build_sns_neurons_report_with_source(&cached_request, &NoLiveSnsNeuronsSource)
        .expect_err("unsupported schema rejected");

    assert!(matches!(
        err,
        SnsHostError::UnsupportedCacheSchemaVersion {
            version: 999,
            expected: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        }
    ));

    let _ = fs::remove_dir_all(root);
}
