use super::*;

#[test]
fn sns_proposals_refresh_writes_complete_cache_and_status_reports_it() {
    let root = temp_dir("ic-query-sns-proposals-refresh");
    let request = sns_proposals_refresh_request(&root, None);

    let refresh = refresh_sns_proposals_cache_with_source(&request, &FixtureSnsProposalsSource)
        .expect("refresh proposals");
    let cache_path = std::path::PathBuf::from(&refresh.cache_path);
    let attempt_path = std::path::PathBuf::from(&refresh.refresh_attempt_path);
    let lock_path = std::path::PathBuf::from(&refresh.refresh_lock_path);

    assert!(cache_path.is_file());
    assert!(attempt_path.is_file());
    assert!(!lock_path.exists());
    assert!(refresh.complete);
    assert_eq!(refresh.page_count, 1);
    assert_eq!(refresh.proposal_count, 1);

    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["schema_version"], 1);
    assert_eq!(cache["domain"], "sns");
    assert_eq!(cache["entity"], ROOT_A);
    assert_eq!(cache["collection"], "proposals");
    assert_eq!(cache["scope"], "full");
    assert_eq!(cache["id"], 1);
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(
        cache["proposals"]
            .as_array()
            .expect("cache proposals")
            .len(),
        1
    );
    assert!(cache.get("metadata").is_none());
    assert!(cache.get("data").is_none());

    let list = build_sns_proposals_cache_list_report(&SnsProposalsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("proposal cache list");
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].id, 1);
    assert_eq!(list.caches[0].cache_status, "ok");
    assert_eq!(list.caches[0].cache_error, None);
    assert_eq!(list.caches[0].row_count, 1);

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);
    assert!(status.found);
    assert_eq!(
        status.cache.as_ref().expect("cache").cache_status.as_str(),
        "ok"
    );
    assert_eq!(
        status
            .cache
            .as_ref()
            .and_then(|cache| cache.latest_attempt.as_ref())
            .map(|attempt| attempt.status.as_str()),
        Some("complete")
    );
    assert!(text.contains("found: yes"));
    assert!(text.contains("row_count: 1"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_missing_cache() {
    let root = temp_dir("ic-query-sns-proposals-cache-missing-status");

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(status.latest_attempt.is_none());
    assert!(text.contains("found: no"));
    assert!(text.contains("refresh_hint: icq sns proposals refresh"));

    let list = build_sns_proposals_cache_list_report(&SnsProposalsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("proposal cache list");
    assert_eq!(list.cache_count, 0);
    assert!(list.caches.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-sns-proposals-status-identity-mismatch");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["entity"] = serde_json::json!("wrong-root");
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_proposals_cache_status(&root, "identity mismatch");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-sns-proposals-status-unsupported-schema");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["schema_version"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_proposals_cache_status(&root, "unsupported SNS cache schema");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-sns-proposals-status-malformed-json");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_sns_proposals_cache_status(&root, "failed to parse SNS cache");

    let _ = fs::remove_dir_all(root);
}
