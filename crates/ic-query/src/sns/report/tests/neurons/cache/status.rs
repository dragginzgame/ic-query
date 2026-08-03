use crate::sns::report::tests::{fixtures::*, *};
use crate::{cache::CacheValidationStatus, test_support::temp_dir};
use std::fs;

#[test]
fn sns_neurons_cache_status_surfaces_malformed_attempt_sidecar() {
    let root = temp_dir("ic-query-sns-neurons-malformed-attempt");
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    fs::create_dir_all(attempt_path.parent().expect("attempt parent"))
        .expect("create attempt parent");
    fs::write(&attempt_path, "{").expect("write malformed attempt");

    let err = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest::new(
        &root,
        MAINNET_NETWORK,
        ROOT_A,
    ))
    .expect_err("malformed attempt must remain visible");

    assert!(matches!(err, SnsHostError::ParseCache { .. }));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_rejects_unknown_attempt_fields() {
    let root = temp_dir("ic-query-sns-neurons-unknown-attempt-field");
    let request = sns_neurons_refresh_request(&root, None);
    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    let mut attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("read attempt"))
            .expect("parse attempt");
    attempt["unexpected"] = serde_json::json!(true);
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&attempt).expect("serialize attempt"),
    )
    .expect("write attempt");

    let err = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest::new(
        &root,
        MAINNET_NETWORK,
        ROOT_A,
    ))
    .expect_err("unknown attempt field must remain visible");

    assert!(matches!(err, SnsHostError::InvalidRefreshAttempt { .. }));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_list_and_status_reports_complete_snapshot() {
    let root = temp_dir("ic-query-sns-neurons-cache-status");
    let request = sns_neurons_refresh_request(&root, None);

    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");

    let list = build_sns_neurons_cache_list_report(&SnsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    })
    .expect("cache list");
    let list_text = sns_neurons_cache_list_report_text(&list);

    assert_eq!(
        list.schema_version,
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION
    );
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].id, 1);
    assert_eq!(list.caches[0].name, "Fixture SNS");
    assert_eq!(list.caches[0].cache_status, CacheValidationStatus::Valid);
    assert_eq!(list.caches[0].cache_error, None);
    assert_eq!(list.caches[0].row_count, 3);
    assert_eq!(list.caches[0].page_count, 3);
    assert!(list.caches[0].complete);
    assert_eq!(
        list.caches[0]
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.status.as_str()),
        Some("complete")
    );
    assert!(list_text.contains("cache_count: 1"));
    assert!(list_text.contains("Fixture SNS"));

    let status = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("cache status");
    let status_text = sns_neurons_cache_status_report_text(&status);

    assert_eq!(
        status.schema_version,
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION
    );
    assert!(status.found);
    assert!(status.expected_cache_path.is_none());
    assert_eq!(
        status.cache.as_ref().expect("cache").cache_status.as_str(),
        "ok"
    );
    assert_eq!(
        status.cache.as_ref().expect("cache").root_canister_id,
        ROOT_A
    );
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.status.as_str()),
        Some("complete")
    );
    assert!(status_text.contains("found: yes"));
    assert!(status_text.contains("cache_path:"));
    assert!(status_text.contains("latest_attempt_status: complete"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-sns-neurons-status-identity-mismatch");
    let cache_path = refresh_fixture_sns_neurons_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["entity"] = serde_json::json!("wrong-root");
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_neurons_cache_status(&root, "identity mismatch");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-sns-neurons-status-unsupported-schema");
    let cache_path = refresh_fixture_sns_neurons_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["schema_version"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_neurons_cache_status(&root, "unsupported SNS cache schema");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_rejects_unknown_fields_and_false_authority() {
    for (field, value, expected_error) in [
        (
            "unexpected",
            serde_json::json!(true),
            "unknown top-level cache field",
        ),
        (
            "point_in_time_guaranteed",
            serde_json::json!(true),
            "point-in-time guarantee",
        ),
    ] {
        let root = temp_dir(&format!("ic-query-sns-neurons-status-{field}"));
        let cache_path = refresh_fixture_sns_neurons_cache(&root);
        let mut cache: serde_json::Value =
            serde_json::from_slice(&fs::read(&cache_path).expect("read cache"))
                .expect("parse cache");
        if field == "unexpected" {
            cache[field] = value;
        } else {
            cache["completeness"][field] = value;
        }
        fs::write(
            &cache_path,
            serde_json::to_vec_pretty(&cache).expect("serialize cache"),
        )
        .expect("write cache");

        assert_invalid_sns_neurons_cache_status(&root, expected_error);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn sns_neurons_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-sns-neurons-status-malformed-json");
    let cache_path = refresh_fixture_sns_neurons_cache(&root);
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_sns_neurons_cache_status(&root, "failed to parse SNS cache");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_reports_inconsistent_row_count() {
    let root = temp_dir("ic-query-sns-neurons-status-row-count");
    let cache_path = refresh_fixture_sns_neurons_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["completeness"]["row_count"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_neurons_cache_status(&root, "actual row count");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_reports_duplicate_neuron_ids() {
    let root = temp_dir("ic-query-sns-neurons-status-duplicate-id");
    let cache_path = refresh_fixture_sns_neurons_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    let duplicate = cache["neurons"][0].clone();
    cache["neurons"]
        .as_array_mut()
        .expect("neuron rows")
        .push(duplicate);
    cache["completeness"]["row_count"] = serde_json::json!(4);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_neurons_cache_status(&root, "duplicate neuron id");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cache_status_reports_failed_attempt_without_complete_cache() {
    let root = temp_dir("ic-query-sns-neurons-cache-failed-status");
    let request = sns_neurons_refresh_request(&root, Some(1));

    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect_err("incomplete refresh");

    let status = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("cache status");
    let status_text = sns_neurons_cache_status_report_text(&status);

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(
        status
            .expected_cache_path
            .as_ref()
            .is_some_and(|path| path.contains(ROOT_A))
    );
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.status.as_str()),
        Some("failed")
    );
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.rows_fetched),
        Some(2)
    );
    assert!(status_text.contains("found: no"));
    assert!(status_text.contains("refresh_hint: icq sns neuron refresh"));
    assert!(status_text.contains("latest_attempt_status: failed"));

    let numeric_status = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("numeric cache status");
    assert!(!numeric_status.found);
    assert_eq!(
        numeric_status
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.status.as_str()),
        Some("failed")
    );
    assert!(numeric_status.expected_cache_path.is_some());

    let _ = fs::remove_dir_all(root);
}

fn refresh_fixture_sns_neurons_cache(root: &std::path::Path) -> std::path::PathBuf {
    let request = sns_neurons_refresh_request(root, None);
    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");
    sns_neurons_cache_path(root, MAINNET_NETWORK, ROOT_A)
}

fn assert_invalid_sns_neurons_cache_status(root: &std::path::Path, expected_error: &str) {
    let status = build_sns_neurons_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.to_path_buf(),
        input: ROOT_A.to_string(),
    })
    .expect("cache status");
    let status_text = sns_neurons_cache_status_report_text(&status);
    let cache = status.cache.as_ref().expect("cache summary");

    assert!(status.found);
    assert_eq!(cache.cache_status, CacheValidationStatus::Invalid);
    assert!(
        cache
            .cache_error
            .as_ref()
            .is_some_and(|error| error.contains(expected_error))
    );
    assert!(status_text.contains("cache_status: invalid"));
    assert!(status_text.contains("cache_error:"));

    let list = build_sns_neurons_cache_list_report(&SnsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.to_path_buf(),
    })
    .expect("cache list");
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].cache_status, CacheValidationStatus::Invalid);
    assert!(
        list.caches[0]
            .cache_error
            .as_ref()
            .is_some_and(|error| error.contains(expected_error))
    );
}
