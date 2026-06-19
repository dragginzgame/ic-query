use super::{
    NNS_PROPOSAL_REPORT_SCHEMA_VERSION, NNS_PROPOSALS_REPORT_SCHEMA_VERSION, NnsProposalHostError,
    model::{
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE, NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
        NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
        NNS_PROPOSAL_STATUS_EXECUTED_CODE, NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
        NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_CODE, NNS_PROPOSAL_VOTE_YES_LABEL, NnsProposalRequest,
        NnsProposalRewardStatusFilter, NnsProposalSortDirection, NnsProposalStatusFilter,
        NnsProposalTopicFilter, NnsProposalsRequest, NnsProposalsSort,
    },
    source::{
        NnsProposalFetchRequest, NnsProposalSource, build_nns_proposal_report_with_source,
        build_nns_proposals_report_with_source,
    },
    text::{nns_proposal_report_text, nns_proposals_report_text},
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
fn nns_proposals_report_filters_sorts_and_renders_rows() {
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
    let request = NnsProposalsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        sort: NnsProposalsSort::Title,
        sort_direction: NnsProposalSortDirection::Asc,
        verbose: true,
    };

    let report =
        build_nns_proposals_report_with_source(&request, &source).expect("build proposals report");
    let text = nns_proposals_report_text(&report);

    assert_eq!(report.schema_version, NNS_PROPOSALS_REPORT_SCHEMA_VERSION);
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
    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
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
    assert!(text.contains("proposal_details:"));
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
    assert!(text.contains("verbose: no"));
    assert!(text.contains(&format!("summary: {}...", "x".repeat(240))));
    assert!(!text.contains(&format!("summary: {}", "x".repeat(260))));
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
