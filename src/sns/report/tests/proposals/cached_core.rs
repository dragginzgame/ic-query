use super::*;

#[test]
fn sns_proposal_detail_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-sns-proposal-detail-cache");
    let refresh_request = sns_proposals_refresh_request(&root, None);
    refresh_sns_proposals_cache_with_source(&refresh_request, &FixtureSnsProposalsSource)
        .expect("refresh proposals cache");
    let mut request = proposal_request("1");
    request.icp_root = Some(root.clone());

    let report = build_sns_proposal_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("cached proposal detail");

    assert_eq!(report.id, 1);
    assert_eq!(report.proposal_id, 42);
    assert_eq!(report.proposal.proposal_id, Some(42));
    assert_eq!(report.proposal.title, "Fixture proposal");
    assert_eq!(report.data_source, "cache");
    assert_eq!(report.cache_complete, Some(true));
    assert!(
        report
            .cache_path
            .as_deref()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );
    let text = sns_proposal_report_text(&report);
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_list_auto_refreshes_missing_cache_and_reuses_it() {
    let root = temp_dir("ic-query-sns-proposals-auto-cache");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = Some(99);
    request.limit = 5;

    let first = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("auto refresh proposals cache");

    assert_eq!(first.proposal_count, 1);
    assert_eq!(first.proposals[0].proposal_id, Some(42));
    assert_eq!(first.data_source, "cache");
    assert_eq!(first.cache_complete, Some(true));
    assert!(
        first
            .cache_path
            .as_deref()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("proposal cache status");
    assert!(status.found);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("cached proposals report");

    assert_eq!(second.proposal_count, 1);
    assert_eq!(second.proposals[0].proposal_id, Some(42));
    assert_eq!(second.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(second.data_source, "cache");
    assert_eq!(second.cache_complete, Some(true));
    assert_eq!(second.cache_path, first.cache_path);
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));

    let _ = fs::remove_dir_all(root);
}
