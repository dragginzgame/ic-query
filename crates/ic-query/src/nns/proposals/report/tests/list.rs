use super::*;

#[test]
fn nns_proposal_list_report_filters_sorts_and_renders_rows() {
    let source = FixtureSource {
        expected_status: vec![NNS_PROPOSAL_STATUS_EXECUTED_CODE],
        expected_reward_status: vec![NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE],
        proposals: vec![
            proposal_info(
                101,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                "Bravo",
                20,
            ),
            proposal_info(
                102,
                NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                "Alpha",
                10,
            ),
        ],
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Bravo",
            20,
        ),
    };
    let request = NnsProposalListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        proposer_neuron_id: Some(99),
        query: None,
        sort: NnsProposalListSort::Title,
        sort_direction: NnsProposalSortDirection::Asc,
        verbose: true,
    };

    let report = build_nns_proposal_list_report_with_source(&request, &source)
        .expect("build proposals report");
    let text = nns_proposal_list_report_text(&report);

    assert_eq!(
        report.schema_version,
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.status_filter, NNS_PROPOSAL_STATUS_EXECUTED_LABEL);
    assert_eq!(
        report.reward_status_filter,
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL
    );
    assert_eq!(report.topic_filter, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL);
    assert_eq!(report.proposer_filter, Some(99));
    assert_eq!(report.query_filter, None);
    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
    assert_eq!(report.result_scope, "bounded-live");
    assert_eq!(report.data_source, "live");
    assert!(report.cache_path.is_none());
    assert!(report.cache_complete.is_none());
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(101));
    assert!(text.contains(&format!(
        "status_filter: {NNS_PROPOSAL_STATUS_EXECUTED_LABEL}"
    )));
    assert!(text.contains(&format!(
        "reward_status_filter: {NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL}"
    )));
    assert!(text.contains(&format!(
        "topic_filter: {NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL}"
    )));
    assert!(text.contains("proposer_filter: 99"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains("proposal_details:"));
}

#[test]
fn nns_proposal_list_report_filters_by_proposer() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: vec![
            proposal_info_with_proposer(101, 99),
            proposal_info_with_proposer(102, 100),
            proposal_info_with_proposer(103, 99),
        ],
        proposal: proposal_info_with_proposer(101, 99),
    };
    let mut request = proposal_sort_request(NnsProposalListSort::Id);
    request.proposer_neuron_id = Some(99);

    let report = build_nns_proposal_list_report_with_source(&request, &source)
        .expect("build proposer-filtered report");
    let text = nns_proposal_list_report_text(&report);

    assert_eq!(report.proposer_filter, Some(99));
    assert_eq!(proposal_ids(&report), vec![103, 101]);
    assert!(text.contains("proposer_filter: 99"));
}

#[test]
fn nns_proposal_list_report_filters_by_query() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: vec![
            proposal_info(
                101,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                "Subnet upgrade",
                20,
            ),
            proposal_info(
                102,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                "Node provider reward",
                10,
            ),
        ],
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Subnet upgrade",
            20,
        ),
    };
    let mut request = proposal_sort_request(NnsProposalListSort::Id);
    request.query = Some("subnet".to_string());

    let report = build_nns_proposal_list_report_with_source(&request, &source)
        .expect("build query-filtered report");
    let text = nns_proposal_list_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize NNS proposal list report");

    assert_eq!(report.query_filter.as_deref(), Some("subnet"));
    assert_eq!(proposal_ids(&report), vec![101]);
    assert_eq!(json["query_filter"], "subnet");
    assert_eq!(json["result_scope"], "bounded-live");
    assert!(text.contains("query_filter: subnet"));
    assert!(text.contains("result_scope: bounded-live"));
}

#[test]
fn nns_proposal_query_filter_matches_searchable_text_fields() {
    let proposal = nns_proposal_row_from_info(proposal_info_with_summary(
        101,
        NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
        NNS_PROPOSAL_STATUS_EXECUTED_CODE,
        "Subnet upgrade",
        20,
        "Committee review",
    ));

    assert!(proposal_matches_query(&proposal, Some("subnet")));
    assert!(proposal_matches_query(&proposal, Some("MOTION")));
    assert!(proposal_matches_query(&proposal, Some("committee")));
    assert!(proposal_matches_query(&proposal, Some("dashboard")));
    assert!(!proposal_matches_query(&proposal, Some("treasury")));
    assert!(proposal_matches_query(&proposal, None));
}
