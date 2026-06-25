use super::*;

#[test]
fn sns_proposals_cached_status_decided_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-status-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(proposal_ids(&second), vec![30]);
    assert!(
        second
            .proposals
            .iter()
            .all(|proposal| proposal.decision_state == SNS_PROPOSAL_DECISION_DECIDED)
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("status_filter: decided"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_status_adopted_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Adopted, &[30]);
}

#[test]
fn sns_proposals_cached_status_rejected_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Rejected, &[20]);
}

#[test]
fn sns_proposals_cached_eligible_filters_complete_snapshot() {
    assert_cached_eligibility_filter(SnsProposalEligibilityFilter::Yes, &[30, 20]);
}

#[test]
fn sns_proposals_cached_ineligible_filters_complete_snapshot() {
    assert_cached_eligibility_filter(SnsProposalEligibilityFilter::No, &[10]);
}

#[test]
fn sns_proposals_cached_proposer_filters_complete_snapshot() {
    assert_cached_proposer_filter("aaaa", &[30]);
}

#[test]
fn sns_proposals_cached_query_filters_complete_snapshot() {
    assert_cached_query_filter("alpha", &[20]);
}

#[test]
fn sns_proposals_status_filter_refreshes_legacy_cache_without_raw_status() {
    let root = temp_dir("ic-query-sns-proposals-status-legacy");
    let refresh = refresh_sns_proposals_cache_with_source(
        &sns_proposals_refresh_request(&root, None),
        &UnsortedSnsProposalsSource,
    )
    .expect("refresh proposals cache");
    let cache_path = std::path::PathBuf::from(refresh.cache_path);
    remove_cached_proposal_status_fields(&cache_path);

    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Adopted;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let report = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("refresh legacy proposals cache before adopted filter");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.status_filter, "adopted");
    assert_eq!(proposal_ids(&report), vec![30]);
    assert_cached_proposal_status_fields_present(&cache_path);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_topic_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-topic-governance");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh topic-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30, 10]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse topic-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30, 10]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_decided_status_combines_with_topic_filter() {
    let root = temp_dir("ic-query-sns-proposals-topic-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided topic-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided topic-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_topic_filter_refreshes_legacy_cache_without_topic() {
    let root = temp_dir("ic-query-sns-proposals-topic-legacy");
    let refresh = refresh_sns_proposals_cache_with_source(
        &sns_proposals_refresh_request(&root, None),
        &UnsortedSnsProposalsSource,
    )
    .expect("refresh proposals cache");
    let cache_path = std::path::PathBuf::from(refresh.cache_path);
    remove_cached_proposal_field(&cache_path, "topic");

    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let report = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("refresh legacy proposals cache before topic filter");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.topic_filter, "governance");
    assert_eq!(proposal_ids(&report), vec![30, 10]);
    assert_cached_proposal_field_present(&cache_path, "topic");

    let _ = fs::remove_dir_all(root);
}
