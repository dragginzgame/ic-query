use super::*;

#[tokio::test(flavor = "current_thread")]
async fn sns_proposal_detail_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-sns-proposal-detail-cache");
    let refresh_request = sns_proposals_refresh_request(&root, None);
    refresh_sns_proposals_cache_with_source(&refresh_request, &FixtureSnsProposalsSource)
        .await
        .expect("refresh proposals cache");
    let mut request = proposal_request(ROOT_A);
    request = request.with_cache_root(root.clone());

    let report = build_sns_proposal_report_with_source(&request, &NoLiveSnsProposalsSource)
        .await
        .expect("cached proposal detail");

    assert_eq!(report.context.instance.id, 1);
    assert_eq!(report.proposal_id, 42);
    assert_eq!(report.proposal.proposal_id, 42);
    assert_eq!(report.proposal.title, "Fixture proposal");
    assert_eq!(report.provenance.data_source().as_str(), "cache");
    assert_eq!(report.provenance.cache_complete(), Some(true));
    assert!(
        report
            .provenance
            .cache_path()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );

    let cache: serde_json::Value = serde_json::from_slice(
        &fs::read(report.provenance.cache_path().unwrap()).expect("read complete snapshot"),
    )
    .unwrap();
    let detail_json = serde_json::to_value(&report).unwrap();
    let mut list_request = proposals_request(ROOT_A);
    list_request = list_request.with_cache_root(root.clone());
    let list = build_sns_proposals_report_with_source_and_progress(
        &list_request,
        &NoLiveSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("cached proposal listing");
    let list_json = serde_json::to_value(&list).unwrap();
    for field in [
        "network",
        "sns_wasm_canister_id",
        "fetched_at",
        "source_endpoint",
        "fetched_by",
        "id",
        "name",
        "root_canister_id",
        "governance_canister_id",
    ] {
        let expected = match field {
            "sns_wasm_canister_id"
            | "id"
            | "name"
            | "root_canister_id"
            | "governance_canister_id" => &cache["metadata"][field],
            _ => &cache[field],
        };
        assert_eq!(&detail_json[field], expected, "detail {field}");
        assert_eq!(&list_json[field], expected, "listing {field}");
    }

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn cached_proposal_reports_reject_non_mainnet_before_cache_lookup() {
    let root = temp_dir("ic-query-sns-proposals-cached-non-mainnet");
    let mainnet_path = refresh_fixture_sns_proposals_cache(&root).await;
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(mainnet_path).expect("read mainnet cache"))
            .expect("parse mainnet cache");
    cache["network"] = serde_json::json!("local");
    let local_path = sns_proposals_cache_path(&root, "local", ROOT_A);
    fs::create_dir_all(local_path.parent().expect("local cache parent"))
        .expect("create local cache parent");
    fs::write(
        &local_path,
        serde_json::to_vec_pretty(&cache).expect("serialize local cache"),
    )
    .expect("write local cache");

    let mut detail_request = proposal_request(ROOT_A);
    detail_request.lookup.network = "local".to_string();
    detail_request.cache_root = Some(root.clone());
    let detail_error =
        build_sns_proposal_report_with_source(&detail_request, &NoLiveSnsProposalsSource)
            .await
            .expect_err("cached detail must reject non-mainnet");

    let mut list_request = proposals_request(ROOT_A);
    list_request.lookup.network = "local".to_string();
    list_request = list_request.with_cache_root(root.clone());
    let list_error = build_sns_proposals_report_with_source_and_progress(
        &list_request,
        &NoLiveSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect_err("cached list must reject non-mainnet");

    assert!(matches!(
        detail_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert!(matches!(
        list_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_list_auto_refreshes_missing_cache_and_reuses_it() {
    let root = temp_dir("ic-query-sns-proposals-auto-cache");
    let mut request = proposals_request(ROOT_A);
    request = request.with_cache_root(root.clone());
    request = request.with_status(SnsProposalStatusFilter::Any).unwrap();
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = Some(99);
    request.limit = 5;

    let first = build_sns_proposals_report_with_source_and_progress(
        &request,
        &FixtureSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("auto refresh proposals cache");

    assert_eq!(first.proposals.len(), 1);
    assert_eq!(first.proposals[0].proposal_id, 42);
    assert_eq!(first.provenance.data_source().as_str(), "cache");
    assert_eq!(first.provenance.cache_complete(), Some(true));
    assert!(
        first
            .provenance
            .cache_path()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );

    let status = build_sns_proposals_cache_status_report(&SnsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    assert!(status.found);

    let second = build_sns_proposals_report_with_source_and_progress(
        &request,
        &NoLiveSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("cached proposals report");

    assert_eq!(second.proposals.len(), 1);
    assert_eq!(second.proposals[0].proposal_id, 42);
    assert_eq!(
        second.context.instance.source_endpoint,
        DEFAULT_SNS_SOURCE_ENDPOINT
    );
    assert_eq!(second.provenance.data_source().as_str(), "cache");
    assert_eq!(second.provenance.cache_complete(), Some(true));
    assert_eq!(
        second.provenance.cache_path(),
        first.provenance.cache_path()
    );

    let _ = fs::remove_dir_all(root);
}
