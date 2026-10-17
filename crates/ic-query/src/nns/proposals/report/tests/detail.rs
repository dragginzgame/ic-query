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
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: Vec::new(),
        proposal: unknown_ballot,
    };
    let request = NnsProposalRequest {
        governance: proposal_governance_request(1_700_000_000),
        proposal_id: 101,
        show_ballots: true,
        verbose: true,
    };

    let report = build_nns_proposal_report_with_source(&request, &source)
        .await
        .expect("build proposal report");
    let text = nns_proposal_report_text(&report);

    assert_eq!(
        report.context.schema_version,
        NNS_PROPOSAL_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.proposal_id, 101);
    assert_eq!(report.data_source.as_str(), "live");
    assert!(report.cache_path.is_none());
    assert!(report.cache_complete.is_none());
    assert!(report.show_ballots);
    assert!(report.verbose);
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
    assert!(text.contains("unknown(99)"));
    let json = serde_json::to_value(&report).expect("serialize proposal");
    assert_eq!(json["proposal"]["ballots"][1]["vote"], 99);
    assert!(text.contains("action: motion"));
    assert!(text.contains("latest_tally_yes: 20"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("verbose: yes"));
    assert!(text.contains("ballots:"));
    assert!(text.contains("NEURON_ID"));
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
        show_ballots: false,
        verbose: false,
    };

    let report = build_nns_proposal_report_with_source(&request, &source)
        .await
        .expect("build proposal report");
    let text = nns_proposal_report_text(&report);

    assert!(!report.verbose);
    assert_eq!(report.data_source.as_str(), "live");
    assert!(text.contains("verbose: no"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains(&format!("summary: {}...", "x".repeat(240))));
    assert!(!text.contains(&format!("summary: {}", "x".repeat(260))));
}
