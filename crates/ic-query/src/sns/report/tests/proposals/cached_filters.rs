use super::*;

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_status_decided_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-status-decided");
    let mut request = proposals_request(ROOT_A);
    request.cache_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .await
        .expect("auto refresh decided proposals cache");

    assert_eq!(first.data_source.as_str(), "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .await
        .expect("reuse decided proposals cache");

    assert_eq!(second.data_source.as_str(), "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(proposal_ids(&second), vec![30]);
    assert!(
        second
            .proposals
            .iter()
            .all(|proposal| proposal.decision_state == SnsProposalDecisionState::Decided)
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("status_filter: decided"));

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_status_adopted_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Adopted, &[30]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_status_rejected_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Rejected, &[20]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_eligible_filters_complete_snapshot() {
    assert_cached_eligibility_filter(SnsProposalEligibilityFilter::Yes, &[30, 20]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_ineligible_filters_complete_snapshot() {
    assert_cached_eligibility_filter(SnsProposalEligibilityFilter::No, &[10]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_proposer_filters_complete_snapshot() {
    assert_cached_proposer_filter("aaaa", &[30]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_query_filters_complete_snapshot() {
    assert_cached_query_filter("AlPhA", &[20]).await;
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_topic_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-topic-governance");
    let mut request = proposals_request(ROOT_A);
    request.cache_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .await
        .expect("auto refresh topic-filtered proposals cache");

    assert_eq!(first.data_source.as_str(), "cache");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30, 10]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .await
        .expect("reuse topic-filtered proposals cache");

    assert_eq!(second.data_source.as_str(), "cache");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30, 10]);

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_cached_decided_status_combines_with_topic_filter() {
    let root = temp_dir("ic-query-sns-proposals-topic-decided");
    let mut request = proposals_request(ROOT_A);
    request.cache_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .await
        .expect("auto refresh decided topic-filtered proposals cache");

    assert_eq!(first.data_source.as_str(), "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .await
        .expect("reuse decided topic-filtered proposals cache");

    assert_eq!(second.data_source.as_str(), "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30]);

    let _ = fs::remove_dir_all(root);
}
