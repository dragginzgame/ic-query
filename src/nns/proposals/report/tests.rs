use super::{
    NNS_PROPOSAL_REPORT_SCHEMA_VERSION, NNS_PROPOSALS_REPORT_SCHEMA_VERSION, NnsProposalHostError,
    model::{
        NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        NnsProposalRequest, NnsProposalSortDirection, NnsProposalStatusFilter,
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
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        assert_eq!(limit, 50);
        assert_eq!(before_proposal_id, Some(200));
        assert_eq!(include_status, self.expected_status);
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
        expected_status: vec![4],
        proposals: vec![
            proposal_info(101, 4, 4, "Bravo", 20),
            proposal_info(102, 7, 4, "Alpha", 10),
        ],
        proposal: proposal_info(101, 4, 4, "Bravo", 20),
    };
    let request = NnsProposalsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Executed,
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
    assert_eq!(report.topic_filter, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL);
    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(101));
    assert!(text.contains(&format!(
        "status_filter: {NNS_PROPOSAL_STATUS_EXECUTED_LABEL}"
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
        proposals: Vec::new(),
        proposal: proposal_info(101, 4, 4, "Bravo", 20),
    };
    let request = NnsProposalRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        proposal_id: 101,
    };

    let report =
        build_nns_proposal_report_with_source(&request, &source).expect("build proposal report");
    let text = nns_proposal_report_text(&report);

    assert_eq!(report.schema_version, NNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.proposal_id, 101);
    assert_eq!(report.proposal.title.as_deref(), Some("Bravo"));
    assert!(text.contains("action: motion"));
    assert!(text.contains("latest_tally_yes: 20"));
}

fn proposal_info(
    proposal_id: u64,
    topic: i32,
    status: i32,
    title: &str,
    yes: u64,
) -> NnsProposalInfo {
    NnsProposalInfo {
        id: Some(NnsProposalId { id: proposal_id }),
        status,
        topic,
        ballots: vec![(
            1,
            NnsGovernanceBallot {
                vote: 1,
                voting_power: yes,
            },
        )],
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
            summary: "Proposal summary".to_string(),
        }),
        proposer: Some(NnsNeuronId { id: 99 }),
        executed_timestamp_seconds: 1_700_000_300,
        total_potential_voting_power: Some(100),
    }
}
