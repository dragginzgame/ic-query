use super::*;
use crate::sns::report::SnsProposalRow;

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_refresh_rejects_invalid_public_page_size() {
    let root = temp_dir("ic-query-sns-proposals-invalid-page-size");
    let mut request = sns_proposals_refresh_request(&root, None);
    request.page_size = 0;

    let err = refresh_sns_proposals_cache_with_source(&request, &FixtureSnsProposalsSource)
        .await
        .expect_err("zero page size is invalid");

    assert!(matches!(
        err,
        SnsHostError::InvalidRefreshPageSize { page_size: 0, .. }
    ));
    let _ = fs::remove_dir_all(root);
}

struct PagedSnsProposalsSource;

delegate_sns_discovery!(PagedSnsProposalsSource);

impl SnsProposalsSource for PagedSnsProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &'a [i32],
        _topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move { unreachable!("refresh does not use bounded proposal fetch") })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(limit, 1);
            assert_eq!(before_proposal_id, None);
            Ok(vec![fixture_proposal_row()])
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_failed_refresh_preserves_page_progress() {
    let root = temp_dir("ic-query-sns-proposals-failed-progress");
    let mut request = sns_proposals_refresh_request(&root, Some(1));
    request.page_size = 1;

    let err = refresh_sns_proposals_cache_with_source(&request, &PagedSnsProposalsSource)
        .await
        .expect_err("capped refresh is incomplete");

    assert!(matches!(
        err,
        SnsHostError::IncompleteRefresh {
            pages_fetched: 1,
            rows_fetched: 1,
            ..
        }
    ));
    let attempt_path = sns_proposals_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read failed attempt"))
            .expect("parse failed attempt");
    assert_eq!(attempt["status"], "failed");
    assert_eq!(attempt["pages_fetched"], 1);
    assert_eq!(attempt["rows_fetched"], 1);
    assert_eq!(attempt["last_cursor"], "42");

    let status = build_sns_proposals_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("failed-attempt status by root");
    assert!(!status.found);
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.status.as_str()),
        Some("failed")
    );
    assert!(status.expected_cache_path.is_some());
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_refresh_writes_complete_cache_and_status_reports_it() {
    let root = temp_dir("ic-query-sns-proposals-refresh");
    let request = sns_proposals_refresh_request(&root, None);

    let refresh = refresh_sns_proposals_cache_with_source(&request, &FixtureSnsProposalsSource)
        .await
        .expect("refresh proposals");
    let cache_path = std::path::PathBuf::from(&refresh.cache_path);
    let attempt_path = std::path::PathBuf::from(&refresh.refresh_attempt_path);
    let lock_path = std::path::PathBuf::from(&refresh.refresh_lock_path);

    assert!(cache_path.is_file());
    assert!(attempt_path.is_file());
    assert!(!crate::cache_file::inspect_refresh_lock(&root, &(lock_path)).unwrap());
    assert!(refresh.complete);
    assert_eq!(refresh.collection, "proposals");
    let refresh_json = serde_json::to_value(&refresh).expect("serialize refresh result");
    assert_eq!(refresh_json["schema_version"], 1);
    assert_eq!(refresh_json["collection"], "proposals");
    assert_eq!(refresh_json["row_count"], refresh.row_count);
    assert_eq!(refresh.page_count, 1);
    assert_eq!(refresh.row_count, 1);

    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["schema_version"], 1);
    assert_eq!(cache["domain"], "sns");
    assert_eq!(cache["entity"], ROOT_A);
    assert_eq!(cache["collection"], "proposals");
    assert_eq!(cache["scope"], "full");
    assert_eq!(cache["metadata"]["id"], 1);
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(
        cache["data"]["proposals"]
            .as_array()
            .expect("cache proposals")
            .len(),
        1
    );

    let list = build_sns_proposals_cache_list_report(&SnsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    })
    .expect("proposal cache list");
    let list_text = sns_proposals_cache_list_report_text(&list);
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].id, 1);
    assert_eq!(list.caches[0].cache_status, CacheValidationStatus::Valid);
    assert_eq!(list.caches[0].cache_error, None);
    assert_eq!(list.caches[0].row_count, 1);
    assert!(list_text.contains("COMPLETE"));

    let status = build_sns_proposals_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);
    assert!(status.found);
    assert_eq!(
        status.cache.as_ref().unwrap().latest_attempt.as_ref(),
        status.latest_attempt.as_ref()
    );
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
    assert_eq!(text.matches("refresh_attempt_path:").count(), 1);
    assert!(text.contains("\n\nlatest_attempt:\n  status: complete"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_missing_cache() {
    let root = temp_dir("ic-query-sns-proposals-cache-missing-status");

    let status = build_sns_proposals_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(status.latest_attempt.is_none());
    assert!(text.contains("found: no"));
    assert!(text.contains("refresh_hint: icq sns proposal refresh"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_surfaces_malformed_attempt_sidecar() {
    let root = temp_dir("ic-query-sns-proposals-malformed-attempt");
    let attempt_path = sns_proposals_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    crate::cache_file::write_managed_text_atomically(&root, &attempt_path, "{")
        .expect("write malformed attempt");

    let err = build_sns_proposals_cache_status_report(&SnsCacheStatusRequest::new(
        &root,
        MAINNET_NETWORK,
        ROOT_A,
    ))
    .expect_err("malformed attempt must remain visible");

    assert!(matches!(
        err,
        SnsHostError::Cache(crate::HostCacheError::ParseCache { .. })
    ));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_cache_list_and_status_reject_malformed_attempts() {
    let root = temp_dir("ic-query-sns-proposals-cached-malformed-attempt");
    refresh_fixture_sns_proposals_cache(&root).await;
    let attempt_path = sns_proposals_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    fs::write(&attempt_path, "{").unwrap();

    assert!(matches!(
        build_sns_proposals_cache_status_report(&SnsCacheStatusRequest::new(
            &root,
            MAINNET_NETWORK,
            ROOT_A,
        )),
        Err(SnsHostError::Cache(
            crate::HostCacheError::ParseCache { .. }
        ))
    ));
    let request = SnsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    };
    assert!(matches!(
        build_sns_proposals_cache_list_report(&request),
        Err(SnsHostError::Cache(
            crate::HostCacheError::ParseCache { .. }
        ))
    ));
    fs::remove_file(attempt_path).unwrap();
    let list = build_sns_proposals_cache_list_report(&request).unwrap();
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].cache_status, CacheValidationStatus::Valid);
    assert!(list.caches[0].latest_attempt.is_none());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-sns-proposals-status-identity-mismatch");
    let cache_path = refresh_fixture_sns_proposals_cache(&root).await;
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

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-sns-proposals-status-unsupported-schema");
    let cache_path = refresh_fixture_sns_proposals_cache(&root).await;
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

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cache_status_rejects_inconsistent_proposal_evidence() {
    for (case, mutate, expected_error) in [
        (
            "action",
            (|cache: &mut serde_json::Value| {
                cache["data"]["proposals"][0]["action_id"] = serde_json::json!("invalid");
            }) as fn(&mut serde_json::Value),
            "invalid type",
        ),
        (
            "vote",
            |cache: &mut serde_json::Value| {
                cache["data"]["proposals"][0]["ballots"][0]["vote"] = serde_json::json!("invalid");
            },
            "invalid type",
        ),
        (
            "ballot-count",
            |cache: &mut serde_json::Value| {
                cache["data"]["proposals"][0]["ballot_count"] = serde_json::json!(2);
            },
            "ballot_count 2 does not match 1 ballot rows",
        ),
    ] {
        let root = temp_dir(&format!("ic-query-sns-proposals-status-{case}"));
        let cache_path = refresh_fixture_sns_proposals_cache(&root).await;
        let mut cache: serde_json::Value =
            serde_json::from_slice(&fs::read(&cache_path).expect("read cache"))
                .expect("parse cache");
        mutate(&mut cache);
        fs::write(
            &cache_path,
            serde_json::to_vec_pretty(&cache).expect("serialize cache"),
        )
        .expect("write cache");

        assert_invalid_sns_proposals_cache_status(&root, expected_error);
        let _ = fs::remove_dir_all(root);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cache_status_rejects_unknown_fields_and_false_authority() {
    for (field, value, expected_error) in [
        ("unexpected", serde_json::json!(true), "unknown field"),
        (
            "point_in_time_guaranteed",
            serde_json::json!(true),
            "point-in-time guarantee",
        ),
    ] {
        let root = temp_dir(&format!("ic-query-sns-proposals-status-{field}"));
        let cache_path = refresh_fixture_sns_proposals_cache(&root).await;
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

        assert_invalid_sns_proposals_cache_status(&root, expected_error);
        let _ = fs::remove_dir_all(root);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-sns-proposals-status-malformed-json");
    let cache_path = refresh_fixture_sns_proposals_cache(&root).await;
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_sns_proposals_cache_status(&root, "failed to parse SNS cache");

    let _ = fs::remove_dir_all(root);
}

struct SecondSnsProposalsSource;

impl SnsDiscoverySource for SecondSnsProposalsSource {
    fn fetch_sns_inventory<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>> {
        Box::pin(async move {
            UnsortedFixtureSnsDiscoverySource
                .fetch_sns_inventory(request)
                .await
        })
    }

    fn fetch_sns_metadata<'a>(
        &'a self,
        request: &'a SnsSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>> {
        Box::pin(async move {
            UnsortedFixtureSnsDiscoverySource
                .fetch_sns_metadata(request, targets)
                .await
        })
    }
}

impl SnsProposalsSource for SecondSnsProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &'a [i32],
        _topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move { unreachable!("refresh uses proposal pages") })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        _limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(sns.id, 2);
            assert_eq!(sns.canisters.root_canister_id, ROOT_B);
            assert_eq!(before_proposal_id, None);
            Ok(vec![])
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_refresh_preserves_inventory_id_across_reports_and_files() {
    for input in ["2", ROOT_B] {
        let root = temp_dir("ic-query-sns-second-position-refresh");
        let mut request = sns_proposals_refresh_request(&root, None);
        request.lookup.input = input.to_string();
        let refresh = refresh_sns_proposals_cache_with_source(&request, &SecondSnsProposalsSource)
            .await
            .expect("refresh second SNS");

        assert_eq!(refresh.id, 2);
        assert_eq!(refresh.root_canister_id, ROOT_B);
        assert!(
            !crate::cache_file::inspect_refresh_lock(
                &root,
                std::path::Path::new(&refresh.refresh_lock_path)
            )
            .unwrap()
        );
        let cache: serde_json::Value =
            serde_json::from_slice(&fs::read(&refresh.cache_path).expect("read cache"))
                .expect("parse cache");
        let attempt: serde_json::Value =
            serde_json::from_slice(&fs::read(&refresh.refresh_attempt_path).expect("read attempt"))
                .expect("parse attempt");
        assert_eq!(cache["metadata"]["id"], refresh.id);
        assert_eq!(cache["entity"], ROOT_B);
        assert_eq!(attempt["metadata"]["id"], refresh.id);
        assert_eq!(attempt["metadata"]["root_canister_id"], ROOT_B);
        assert_eq!(attempt["status"], "complete");

        let mut cached_request = proposals_request(ROOT_B);
        cached_request.cache_root = Some(root.clone());
        let report =
            build_sns_proposals_report_with_source(&cached_request, &NoLiveSnsProposalsSource)
                .await
                .expect("read second SNS from cache");
        assert_eq!(report.id, refresh.id);
        assert_eq!(report.root_canister_id, ROOT_B);
        let _ = fs::remove_dir_all(root);
    }
}

struct UnorderedPagedProposalsSource {
    invalid_second_page: bool,
}

delegate_sns_discovery!(UnorderedPagedProposalsSource);

impl SnsProposalsSource for UnorderedPagedProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &'a [i32],
        _topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move { unreachable!("refresh uses pages") })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(limit, 2);
            let ids = match before_proposal_id {
                None => vec![20, 30],
                Some(20) => vec![if self.invalid_second_page { 20 } else { 10 }],
                other => panic!("unexpected continuation {other:?}"),
            };
            Ok(ids
                .into_iter()
                .map(|id| {
                    let mut row = fixture_proposal_row();
                    row.proposal_id = id;
                    row
                })
                .collect())
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn proposal_refresh_uses_lowest_id_and_rejects_boundary_overlap() {
    let root = temp_dir("ic-query-sns-proposals-page-boundary");
    let mut request = sns_proposals_refresh_request(&root, None);
    request.page_size = 2;
    let report = refresh_sns_proposals_cache_with_source(
        &request,
        &UnorderedPagedProposalsSource {
            invalid_second_page: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(report.row_count, 3);
    let path = sns_proposals_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let original = fs::read(&path).unwrap();
    let error = refresh_sns_proposals_cache_with_source(
        &request,
        &UnorderedPagedProposalsSource {
            invalid_second_page: true,
        },
    )
    .await
    .expect_err("exclusive cursor rejects overlap even on a terminal page");
    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS proposal page",
            ..
        }
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    let attempt: serde_json::Value = serde_json::from_slice(
        &fs::read(sns_proposals_refresh_attempt_path(
            &root,
            MAINNET_NETWORK,
            ROOT_A,
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(attempt["status"], "failed");
    assert_eq!(attempt["pages_fetched"], 1);
    fs::remove_dir_all(root).unwrap();
}
