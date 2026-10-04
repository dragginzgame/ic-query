use crate::sns::report::tests::{fixtures::*, *};
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn sns_neurons_refresh_rejects_invalid_public_page_size() {
    let root = temp_dir("ic-query-sns-neurons-invalid-page-size");
    let mut request = sns_neurons_refresh_request(&root, None);
    request.page_size = 0;

    let err = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect_err("zero page size is invalid");

    assert!(matches!(
        err,
        SnsHostError::InvalidRefreshPageSize { page_size: 0, .. }
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cached_sort_requires_existing_complete_cache() {
    let root = temp_dir("ic-query-sns-neurons-missing-cache");
    let mut request = neurons_request("1");
    request.cache_root = Some(root.clone());
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
    assert_eq!(attempt["last_cursor"], "02".repeat(32));
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
    cached_request.cache_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    let err = build_sns_neurons_report_with_source(&cached_request, &NoLiveSnsNeuronsSource)
        .expect_err("unsupported schema rejected");

    assert!(matches!(
        err,
        SnsHostError::Cache(crate::HostCacheError::UnsupportedCacheSchemaVersion {
            version: 999,
            expected: SNS_NEURONS_CACHE_SCHEMA_VERSION,
            ..
        })
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_cached_sort_rejects_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-sns-neurons-identity-mismatch");
    let request = sns_neurons_refresh_request(&root, None);
    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");

    let cache_path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["entity"] = serde_json::json!("wrong-root");
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    let mut cached_request = neurons_request("1");
    cached_request.cache_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    let err = build_sns_neurons_report_with_source(&cached_request, &NoLiveSnsNeuronsSource)
        .expect_err("identity mismatch rejected");

    match err {
        SnsHostError::CacheIdentityMismatch {
            path,
            field,
            expected,
            actual,
        } => {
            assert_eq!(path, cache_path);
            assert_eq!(field, "entity");
            assert_eq!(expected, ROOT_A);
            assert_eq!(actual, "wrong-root");
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[derive(Clone, Copy)]
enum InvalidNeuronPage {
    Cursor,
    DuplicateIds,
    DescendingIds,
}

struct InvalidNeuronPageSource(InvalidNeuronPage);

delegate_sns_discovery!(InvalidNeuronPageSource);

impl SnsNeuronsSource for InvalidNeuronPageSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("refresh uses pages")
    }

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        let mut page = PagedFixtureSnsNeuronsSource.fetch_sns_neuron_page(
            request,
            sns,
            limit,
            start_page_at,
            owner_principal_id,
        )?;
        if start_page_at.is_some() {
            match self.0 {
                InvalidNeuronPage::Cursor => {
                    page.last_cursor = Some(SnsNeuronId { id: vec![4; 32] });
                }
                InvalidNeuronPage::DuplicateIds => {
                    page.neurons[0] = page.neurons[1].clone();
                }
                InvalidNeuronPage::DescendingIds => {
                    page.neurons[0].neuron_id = "04".repeat(32);
                }
            }
        }
        Ok(page)
    }
}

#[test]
fn invalid_neuron_page_preserves_complete_cache_and_records_failure() {
    let root = temp_dir("ic-query-sns-neurons-invalid-cursor");
    let request = sns_neurons_refresh_request(&root, None);
    refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource).unwrap();
    let path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let original = fs::read(&path).unwrap();
    for invalid_page in [
        InvalidNeuronPage::Cursor,
        InvalidNeuronPage::DuplicateIds,
        InvalidNeuronPage::DescendingIds,
    ] {
        let error =
            refresh_sns_neurons_cache_with_source(&request, &InvalidNeuronPageSource(invalid_page))
                .expect_err("invalid page cannot prove completeness");
        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS neuron page",
                ..
            }
        ));
        assert_eq!(fs::read(&path).unwrap(), original);
        let attempt: serde_json::Value = serde_json::from_slice(
            &fs::read(sns_neurons_refresh_attempt_path(
                &root,
                MAINNET_NETWORK,
                ROOT_A,
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(attempt["status"], "failed");
        assert_eq!(attempt["pages_fetched"], 1);
        assert_eq!(attempt["rows_fetched"], 2);
        assert!(!sns_neurons_refresh_lock_path(&root, MAINNET_NETWORK, ROOT_A).exists());
    }
    fs::remove_dir_all(root).unwrap();
}
