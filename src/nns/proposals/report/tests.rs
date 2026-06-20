use super::{
    NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
    NnsProposalHostError,
    labels::{nns_proposal_status_text, nns_reward_status_text, nns_topic_text},
    model::{
        NNS_PROPOSAL_REWARD_STATUS_ACCEPT_VOTES_CODE, NNS_PROPOSAL_REWARD_STATUS_INELIGIBLE_CODE,
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE, NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
        NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_DEADLINE_LABEL,
        NNS_PROPOSAL_SORT_DESC_LABEL, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
        NNS_PROPOSAL_SORT_TALLY_TIME_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
        NNS_PROPOSAL_SORT_VOTING_POWER_LABEL, NNS_PROPOSAL_STATUS_EXECUTED_CODE,
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_STATUS_OPEN_CODE,
        NNS_PROPOSAL_STATUS_OPEN_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL, NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE,
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_CODE, NNS_PROPOSAL_VOTE_YES_LABEL,
        NnsProposalListRequest, NnsProposalListSort, NnsProposalRequest,
        NnsProposalRewardStatusFilter, NnsProposalSortDirection, NnsProposalStatusFilter,
        NnsProposalTopicFilter,
    },
    source::{
        NnsProposalFetchRequest, NnsProposalSource, build_nns_proposal_list_report_with_source,
        build_nns_proposal_report_with_source,
    },
    text::{nns_proposal_list_report_text, nns_proposal_report_text},
    wire::{
        NnsGovernanceBallot, NnsNeuronId, NnsProposal, NnsProposalAction, NnsProposalId,
        NnsProposalInfo, NnsProposalTallyWire,
    },
};
use crate::{
    ic_registry::{DEFAULT_MAINNET_ENDPOINT, MAINNET_GOVERNANCE_CANISTER_ID},
    subnet_catalog::MAINNET_NETWORK,
};
use candid::Reserved;

#[derive(Clone, Debug)]
struct FixtureSource {
    expected_status: Vec<i32>,
    expected_reward_status: Vec<i32>,
    proposals: Vec<NnsProposalInfo>,
    proposal: NnsProposalInfo,
}

impl NnsProposalSource for FixtureSource {
    fn fetch_proposals(
        &self,
        _request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        include_reward_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        assert_eq!(limit, 50);
        assert_eq!(before_proposal_id, Some(200));
        assert_eq!(include_status, self.expected_status);
        assert_eq!(include_reward_status, self.expected_reward_status);
        Ok(self.proposals.clone())
    }

    fn fetch_proposal(
        &self,
        _request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError> {
        assert_eq!(proposal_id, 101);
        Ok(self.proposal.clone())
    }
}

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
    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
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
        sort: NnsProposalListSort::RewardStatus,
        sort_direction: NnsProposalSortDirection::Asc,
        verbose: false,
    };

    let report = build_nns_proposal_list_report_with_source(&request, &source)
        .expect("build reward-status sorted report");

    assert_eq!(report.sort, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
    assert_eq!(
        report
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![102, 103, 101]
    );
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

#[test]
fn nns_proposal_labels_cover_common_values() {
    assert_eq!(
        nns_proposal_status_text(NNS_PROPOSAL_STATUS_OPEN_CODE),
        NNS_PROPOSAL_STATUS_OPEN_LABEL
    );
    assert_eq!(
        nns_proposal_status_text(NNS_PROPOSAL_STATUS_EXECUTED_CODE),
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL
    );
    assert_eq!(
        nns_reward_status_text(NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE),
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL
    );
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE),
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL
    );
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE),
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL
    );
}

fn proposal_sort_request(sort: NnsProposalListSort) -> NnsProposalListRequest {
    NnsProposalListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Any,
        reward_status: NnsProposalRewardStatusFilter::Any,
        topic: NnsProposalTopicFilter::Any,
        proposer_neuron_id: None,
        sort,
        sort_direction: sort.default_direction(),
        verbose: false,
    }
}

fn proposal_ids(report: &super::model::NnsProposalListReport) -> Vec<u64> {
    report
        .proposals
        .iter()
        .filter_map(|proposal| proposal.proposal_id)
        .collect()
}

fn proposal_info(
    proposal_id: u64,
    topic: i32,
    status: i32,
    title: &str,
    yes: u64,
) -> NnsProposalInfo {
    proposal_info_with_summary(proposal_id, topic, status, title, yes, "Proposal summary")
}

fn proposal_info_with_reward_status(
    proposal_id: u64,
    topic: i32,
    status: i32,
    reward_status: i32,
    title: &str,
    yes: u64,
) -> NnsProposalInfo {
    NnsProposalInfo {
        reward_status,
        ..proposal_info(proposal_id, topic, status, title, yes)
    }
}

fn proposal_info_with_deadline_and_voting_power(
    proposal_id: u64,
    deadline_timestamp_seconds: Option<u64>,
    total_potential_voting_power: Option<u64>,
) -> NnsProposalInfo {
    NnsProposalInfo {
        deadline_timestamp_seconds,
        total_potential_voting_power,
        ..proposal_info(
            proposal_id,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Sortable proposal",
            20,
        )
    }
}

fn proposal_info_with_tally_timestamp(
    proposal_id: u64,
    tally_timestamp_seconds: Option<u64>,
) -> NnsProposalInfo {
    let latest_tally = tally_timestamp_seconds.map(|timestamp_seconds| NnsProposalTallyWire {
        timestamp_seconds,
        yes: 20,
        no: 1,
        total: 21,
    });

    NnsProposalInfo {
        latest_tally,
        ..proposal_info(
            proposal_id,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Tally timestamp proposal",
            20,
        )
    }
}

fn proposal_info_with_proposer(proposal_id: u64, proposer_neuron_id: u64) -> NnsProposalInfo {
    NnsProposalInfo {
        proposer: Some(NnsNeuronId {
            id: proposer_neuron_id,
        }),
        ..proposal_info(
            proposal_id,
            NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE,
            NNS_PROPOSAL_STATUS_EXECUTED_CODE,
            "Proposer proposal",
            20,
        )
    }
}

fn proposal_info_with_summary(
    proposal_id: u64,
    topic: i32,
    status: i32,
    title: &str,
    yes: u64,
    summary: &str,
) -> NnsProposalInfo {
    NnsProposalInfo {
        id: Some(NnsProposalId { id: proposal_id }),
        status,
        topic,
        ballots: vec![
            (
                2,
                NnsGovernanceBallot {
                    vote: 2,
                    voting_power: 1,
                },
            ),
            (
                1,
                NnsGovernanceBallot {
                    vote: 1,
                    voting_power: yes,
                },
            ),
        ],
        proposal_timestamp_seconds: 1_700_000_000 + proposal_id,
        reward_event_round: 7,
        deadline_timestamp_seconds: Some(1_700_010_000),
        failed_timestamp_seconds: 0,
        reject_cost_e8s: 100_000_000,
        latest_tally: Some(NnsProposalTallyWire {
            no: 1,
            yes,
            total: yes + 1,
            timestamp_seconds: 1_700_000_100,
        }),
        reward_status: 3,
        decided_timestamp_seconds: 1_700_000_200,
        proposal: Some(NnsProposal {
            url: "https://dashboard.internetcomputer.org/proposal/101".to_string(),
            title: Some(title.to_string()),
            action: Some(NnsProposalAction::Motion(Reserved)),
            summary: summary.to_string(),
        }),
        proposer: Some(NnsNeuronId { id: 99 }),
        executed_timestamp_seconds: 1_700_000_300,
        total_potential_voting_power: Some(100),
    }
}
