use super::*;

#[test]
fn nns_proposal_report_renders_detail() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: Vec::new(),
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Bravo",
            20,
        ),
    };
    let request = NnsProposalRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        proposal_id: 101,
        show_ballots: true,
        verbose: true,
    };

    let report =
        build_nns_proposal_report_with_source(&request, &source).expect("build proposal report");
    let text = nns_proposal_report_text(&report);

    assert_eq!(report.schema_version, NNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.proposal_id, 101);
    assert_eq!(report.data_source, "live");
    assert!(report.cache_path.is_none());
    assert!(report.cache_complete.is_none());
    assert!(report.show_ballots);
    assert!(report.verbose);
    assert_eq!(report.proposal.title.as_deref(), Some("Bravo"));
    assert_eq!(report.proposal.ballots[0].neuron_id, 1);
    assert_eq!(
        report.proposal.ballots[0].vote_text,
        NNS_PROPOSAL_VOTE_YES_LABEL
    );
    assert!(text.contains("action: motion"));
    assert!(text.contains("latest_tally_yes: 20"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("verbose: yes"));
    assert!(text.contains("ballots:"));
    assert!(text.contains("NEURON_ID"));
}

#[test]
fn nns_proposal_report_truncates_summary_without_verbose() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: Vec::new(),
        proposal: proposal_info_with_summary(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Bravo",
            20,
            &"x".repeat(260),
        ),
    };
    let request = NnsProposalRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        proposal_id: 101,
        show_ballots: false,
        verbose: false,
    };

    let report =
        build_nns_proposal_report_with_source(&request, &source).expect("build proposal report");
    let text = nns_proposal_report_text(&report);

    assert!(!report.verbose);
    assert_eq!(report.data_source, "live");
    assert!(text.contains("verbose: no"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains(&format!("summary: {}...", "x".repeat(240))));
    assert!(!text.contains(&format!("summary: {}", "x".repeat(260))));
}
