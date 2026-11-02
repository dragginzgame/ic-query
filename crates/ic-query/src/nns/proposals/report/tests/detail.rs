use super::*;

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_report_renders_detail() {
    let mut unknown_ballot = proposal_info(
        101,
        NnsProposalTopic::Governance.code(),
        NnsProposalStatus::Executed.code(),
        "Bravo",
        20,
    );
    unknown_ballot.ballots[0].1.vote = 99;
    unknown_ballot.proposal_timestamp_seconds = 1_700_000_000;
    unknown_ballot.deadline_timestamp_seconds = Some(0);
    unknown_ballot.decided_timestamp_seconds = 0;
    unknown_ballot.executed_timestamp_seconds = 1_700_000_000;
    unknown_ballot.failed_timestamp_seconds = 0;
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: Vec::new(),
        proposal: unknown_ballot,
    };
    let request = NnsProposalRequest {
        governance: proposal_governance_request(1_700_000_000),
        proposal_id: 101,
    };

    let report = build_nns_proposal_report_with_source(&request, &source)
        .await
        .expect("build proposal report");

    assert_eq!(
        report.context.schema_version,
        NNS_PROPOSAL_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.proposal_id, 101);
    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert!(report.provenance.cache_path().is_none());
    assert!(report.provenance.cache_complete().is_none());

    assert_eq!(report.proposal.title.as_deref(), Some("Bravo"));
    assert_eq!(report.proposal.ballots[0].neuron_id, 1);
    assert_eq!(
        crate::nns::governance::NnsGovernanceVote::from_code(report.proposal.ballots[0].vote),
        crate::nns::governance::NnsGovernanceVote::Yes
    );
    let unknown = &report.proposal.ballots[1];
    assert_eq!(unknown.vote, 99);
    assert_eq!(
        crate::nns::governance::NnsGovernanceVote::from_code(unknown.vote),
        crate::nns::governance::NnsGovernanceVote::Unknown(99)
    );

    let json = serde_json::to_value(&report).expect("serialize proposal");
    assert_eq!(json["proposal"]["ballots"][1]["vote"], 99);
    assert_eq!(
        json["proposal"]["proposal_timestamp_seconds"],
        1_700_000_000
    );
    assert_eq!(json["proposal"]["deadline_timestamp_seconds"], 0);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_report_truncates_summary_without_verbose() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: Vec::new(),
        proposal: proposal_info_with_summary(
            101,
            NnsProposalTopic::Governance.code(),
            NnsProposalStatus::Executed.code(),
            "Bravo",
            20,
            &"x".repeat(260),
        ),
    };
    let request = NnsProposalRequest {
        governance: proposal_governance_request(1_700_000_000),
        proposal_id: 101,
    };

    let report = build_nns_proposal_report_with_source(&request, &source)
        .await
        .expect("build proposal report");

    assert_eq!(report.provenance.data_source().as_str(), "live");
}
