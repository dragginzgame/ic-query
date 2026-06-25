use super::*;

#[test]
fn nns_proposal_list_report_sorts_by_reward_status_text() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: vec![
            proposal_info_with_reward_status(
                101,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE,
                "Settled proposal",
                20,
            ),
            proposal_info_with_reward_status(
                102,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                NNS_PROPOSAL_REWARD_STATUS_ACCEPT_VOTES_CODE,
                "Accept votes proposal",
                10,
            ),
            proposal_info_with_reward_status(
                103,
                NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
                NNS_PROPOSAL_STATUS_EXECUTED_CODE,
                NNS_PROPOSAL_REWARD_STATUS_INELIGIBLE_CODE,
                "Ineligible proposal",
                30,
            ),
        ],
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Settled proposal",
            20,
        ),
    };
    let request = NnsProposalListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Any,
        reward_status: NnsProposalRewardStatusFilter::Any,
        topic: NnsProposalTopicFilter::Any,
        proposer_neuron_id: None,
        query: None,
        sort: NnsProposalListSort::RewardStatus,
        sort_direction: NnsProposalSortDirection::Asc,
        verbose: false,
    };

    let report = build_nns_proposal_list_report_with_source(&request, &source)
        .expect("build reward-status sorted report");

    assert_eq!(report.sort, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
    assert_eq!(proposal_ids(&report), vec![102, 103, 101]);
}

#[test]
fn nns_proposal_list_report_sorts_by_deadline_and_voting_power() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: vec![
            proposal_info_with_deadline_and_voting_power(101, Some(1_700_000_300), Some(200)),
            proposal_info_with_deadline_and_voting_power(102, None, Some(500)),
            proposal_info_with_deadline_and_voting_power(103, Some(1_700_000_100), None),
        ],
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Deadline proposal",
            20,
        ),
    };

    let deadline_report = build_nns_proposal_list_report_with_source(
        &proposal_sort_request(NnsProposalListSort::Deadline),
        &source,
    )
    .expect("build deadline sorted report");

    assert_eq!(deadline_report.sort, NNS_PROPOSAL_SORT_DEADLINE_LABEL);
    assert_eq!(deadline_report.sort_direction, NNS_PROPOSAL_SORT_DESC_LABEL);
    assert_eq!(proposal_ids(&deadline_report), vec![101, 103, 102]);

    let voting_power_report = build_nns_proposal_list_report_with_source(
        &proposal_sort_request(NnsProposalListSort::VotingPower),
        &source,
    )
    .expect("build voting-power sorted report");

    assert_eq!(
        voting_power_report.sort,
        NNS_PROPOSAL_SORT_VOTING_POWER_LABEL
    );
    assert_eq!(
        voting_power_report.sort_direction,
        NNS_PROPOSAL_SORT_DESC_LABEL
    );
    assert_eq!(proposal_ids(&voting_power_report), vec![102, 101, 103]);
}

#[test]
fn nns_proposal_list_report_sorts_by_tally_time() {
    let source = FixtureSource {
        expected_status: Vec::new(),
        expected_reward_status: Vec::new(),
        proposals: vec![
            proposal_info_with_tally_timestamp(101, Some(1_700_000_300)),
            proposal_info_with_tally_timestamp(102, None),
            proposal_info_with_tally_timestamp(103, Some(1_700_000_100)),
        ],
        proposal: proposal_info(
            101,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Tally timestamp proposal",
            20,
        ),
    };

    let report = build_nns_proposal_list_report_with_source(
        &proposal_sort_request(NnsProposalListSort::TallyTime),
        &source,
    )
    .expect("build tally-time sorted report");

    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TALLY_TIME_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_DESC_LABEL);
    assert_eq!(proposal_ids(&report), vec![101, 103, 102]);
}
