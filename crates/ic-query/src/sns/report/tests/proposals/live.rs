use super::*;

#[test]
fn sns_proposal_resolves_list_id_and_renders_governance_proposal() {
    let request = proposal_request("1");

    let report = build_sns_proposal_report_with_source(&request, &FixtureSnsProposalSource)
        .expect("sns proposal report");
    let text = sns_proposal_report_text(&report);

    assert_eq!(report.schema_version, SNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.proposal_id, 42);
    assert!(report.show_ballots);
    assert_eq!(report.proposal.proposal_id, Some(42));
    assert_eq!(report.proposal.action, "motion");
    assert_eq!(report.proposal.decision_state, SNS_PROPOSAL_DECISION_OPEN);
    assert_eq!(report.proposal.ballot_count, 1);
    assert_eq!(report.proposal.ballots[0].vote_text, "yes");
    assert_eq!(report.data_source, "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.cache_complete, None);
    assert!(text.contains("proposal_id: 42"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains("action: motion"));
    assert!(text.contains("ballot_count: 1"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("NEURON_ID   VOTE"));
    assert!(text.contains("00010203"));
    assert!(text.contains("1.00"));
    assert!(text.contains("Fixture proposal"));
    assert!(text.contains("reject_cost: 1.00"));
}

#[test]
fn sns_proposals_resolves_list_id_and_renders_governance_proposals() {
    let request = proposals_request("1");

    let report = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("sns proposals report");
    let text = sns_proposals_report_text(&report);

    assert_eq!(report.schema_version, SNS_PROPOSALS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.requested_limit, 10);
    assert_eq!(report.before_proposal_id, Some(99));
    assert_eq!(report.status_filter, "open");
    assert_eq!(report.topic_filter, "governance");
    assert_eq!(report.eligibility_filter, "any");
    assert_eq!(report.proposer_filter, None);
    assert_eq!(report.query_filter, None);
    assert_eq!(report.sort, "api");
    assert_eq!(report.sort_direction, "none");
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(42));
    assert_eq!(report.proposals[0].action, "motion");
    assert_eq!(
        report.proposals[0].decision_state,
        SNS_PROPOSAL_DECISION_OPEN
    );
    assert_eq!(report.proposals[0].reject_cost_e8s, 100_000_000);
    assert_eq!(report.proposals[0].created_at, "2026-06-01T00:00:00Z");
    assert_eq!(
        report.proposals[0]
            .latest_tally
            .as_ref()
            .map(|tally| tally.yes),
        Some(10)
    );
    assert_eq!(report.data_source, "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.cache_complete, None);
    assert!(text.contains("status_filter: open"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains("topic_filter: governance"));
    assert!(text.contains("eligibility_filter: any"));
    assert!(text.contains("proposer_filter: -"));
    assert!(text.contains("query_filter: -"));
    assert!(text.contains("sort: api"));
    assert!(text.contains("sort_direction: none"));
    assert!(text.contains("before_proposal_id: 99"));
    assert!(text.contains("proposal_count: 1"));
    assert!(text.contains("ID   ACTION"));
    assert!(text.contains("motion"));
    assert!(text.contains("Fixture proposal"));
}

#[test]
fn sns_proposals_live_proposer_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.proposer_neuron_id = Some("000102".to_string());

    let report = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("sns proposals report");

    assert_eq!(report.data_source, "live");
    assert_eq!(report.proposer_filter.as_deref(), Some("000102"));
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(42));
}

#[test]
fn sns_proposals_live_eligibility_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.eligibility = SnsProposalEligibilityFilter::No;

    let report = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("sns proposals report");

    assert_eq!(report.data_source, "live");
    assert_eq!(report.eligibility_filter, "no");
    assert_eq!(report.proposal_count, 0);
    assert!(report.proposals.is_empty());
}

#[test]
fn sns_proposals_live_query_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.query = Some("fixture proposal".to_string());

    let report = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("sns proposals report");
    let text = sns_proposals_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize live SNS proposals report");

    assert_eq!(report.data_source, "live");
    assert_eq!(report.query_filter.as_deref(), Some("fixture proposal"));
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(42));
    assert_eq!(json["query_filter"], "fixture proposal");
    assert!(text.contains("query_filter: fixture proposal"));
}

#[test]
fn sns_proposals_status_decided_requires_cache_root() {
    let mut request = proposals_request("1");
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Governance;

    let error = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect_err("decided without cache rejected");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedProposalView { .. }
    ));

    request.topic = SnsProposalTopicFilter::Any;
    let error = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect_err("decided without cache rejected");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedProposalView { .. }
    ));
}
