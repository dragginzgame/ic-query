use super::super::super::{fixtures::*, *};
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn sns_neurons_refresh_writes_complete_cache_and_cached_sort_uses_it() {
    let root = temp_dir("ic-query-sns-neurons-refresh");
    let request = sns_neurons_refresh_request(&root, None);

    let refresh = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");
    let cache_path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    let lock_path = sns_neurons_refresh_lock_path(&root, MAINNET_NETWORK, ROOT_A);

    assert!(cache_path.is_file());
    assert!(attempt_path.is_file());
    assert!(!lock_path.exists());
    assert!(refresh.complete);
    assert_eq!(refresh.page_count, 3);
    assert_eq!(refresh.neuron_count, 3);

    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["schema_version"], 1);
    assert_eq!(cache["id"], 1);
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(cache["neurons"].as_array().expect("cache neurons").len(), 3);
    assert!(cache.get("metadata").is_none());
    assert!(cache.get("data").is_none());

    let mut cached_request = neurons_request("1");
    cached_request.icp_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    cached_request.limit = 2;
    let report = build_sns_neurons_report_with_source(&cached_request, &NoLiveSnsNeuronsSource)
        .expect("cached neurons report");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.sort, "stake");
    assert_eq!(report.total_neuron_count, 3);
    assert_eq!(report.neuron_count, 2);
    assert_eq!(report.neurons[0].neuron_id, "03");
    assert_eq!(report.neurons[0].cached_neuron_stake_e8s, 50);
    assert_eq!(report.neurons[1].neuron_id, "02");
    assert_eq!(report.neurons[1].cached_neuron_stake_e8s, 30);

    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "complete");
    assert_eq!(attempt["root_canister_id"], ROOT_A);
    assert!(attempt.get("metadata").is_none());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_refresh_failure_preserves_existing_complete_cache() {
    let root = temp_dir("ic-query-sns-neurons-refresh-preserves-cache");
    let complete_request = sns_neurons_refresh_request(&root, None);
    refresh_sns_neurons_cache_with_source(&complete_request, &PagedFixtureSnsNeuronsSource)
        .expect("complete refresh");

    let failed_request = sns_neurons_refresh_request(&root, Some(1));
    refresh_sns_neurons_cache_with_source(&failed_request, &PagedFixtureSnsNeuronsSource)
        .expect_err("incomplete refresh");

    let mut cached_request = neurons_request("1");
    cached_request.icp_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    let report = build_sns_neurons_report_with_source(&cached_request, &NoLiveSnsNeuronsSource)
        .expect("previous complete cache remains usable");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.total_neuron_count, 3);
    assert_eq!(report.neurons[0].neuron_id, "03");

    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "failed");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_refresh_removes_stale_lock_and_publishes_cache() {
    let root = temp_dir("ic-query-sns-neurons-stale-lock");
    let request = sns_neurons_refresh_request(&root, None);
    let cache_path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let lock_path = sns_neurons_refresh_lock_path(&root, MAINNET_NETWORK, ROOT_A);
    fs::create_dir_all(lock_path.parent().expect("lock path parent")).expect("create cache dir");
    fs::write(
        &lock_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "network": MAINNET_NETWORK,
            "pid": 999_999,
            "started_at_unix_ms": 1,
            "target_path": cache_path.display().to_string(),
        }))
        .expect("serialize stale lock"),
    )
    .expect("write stale lock");

    let refresh = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh with stale lock");

    assert!(refresh.complete);
    assert!(cache_path.is_file());
    assert!(!lock_path.exists());

    let _ = fs::remove_dir_all(root);
}
