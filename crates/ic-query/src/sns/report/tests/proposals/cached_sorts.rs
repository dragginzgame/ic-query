use super::*;

#[test]
fn sns_proposals_cached_sort_created_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-created");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Created;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "created");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![20, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "created");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![20, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: created"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_created_ascending_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-created-asc");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Created;
    request.sort_direction = SnsProposalSortDirection::Asc;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh ascending sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "created");
    assert_eq!(first.sort_direction, "asc");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse ascending sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "created");
    assert_eq!(second.sort_direction, "asc");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: created"));
    assert!(text.contains("sort_direction: asc"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_decided_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Decided;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "decided");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "decided");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: decided"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_executed_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-executed");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Executed;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh executed sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "executed");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse executed sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "executed");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: executed"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_failed_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-failed");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Failed;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh failed sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "failed");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse failed sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "failed");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: failed"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_title_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Title,
        SnsProposalSortDirection::Asc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_status_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Status,
        SnsProposalSortDirection::Asc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_topic_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Topic,
        SnsProposalSortDirection::Asc,
        &[10, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_proposer_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Proposer,
        SnsProposalSortDirection::Asc,
        &[30, 10],
    );
}

#[test]
fn sns_proposals_cached_sort_action_id_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::ActionId,
        SnsProposalSortDirection::Desc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_total_votes_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::TotalVotes,
        SnsProposalSortDirection::Desc,
        &[10, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_tally_time_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::TallyTime,
        SnsProposalSortDirection::Desc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_reward_round_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::RewardRound,
        SnsProposalSortDirection::Desc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_reward_end_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::RewardEnd,
        SnsProposalSortDirection::Desc,
        &[30, 10],
    );
}

#[test]
fn sns_proposals_cached_sort_eligible_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Eligible,
        SnsProposalSortDirection::Desc,
        &[30, 20],
    );
}

#[test]
fn sns_proposals_cached_sort_reject_cost_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::RejectCost,
        SnsProposalSortDirection::Desc,
        &[30, 10],
    );
}
