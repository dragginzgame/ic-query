use super::{
    NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_MAX_PAGE_SIZE,
    NNS_PROPOSAL_REPORT_SCHEMA_VERSION, NnsProposalError,
    model::{
        NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort, NnsProposalRequest,
        NnsProposalRewardStatus, NnsProposalRewardStatusFilter, NnsProposalRow,
        NnsProposalSortDirection, NnsProposalStatus, NnsProposalStatusFilter, NnsProposalTopic,
        NnsProposalTopicFilter, NnsProposalVote,
        selection::{
            NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_DEADLINE_LABEL,
            NNS_PROPOSAL_SORT_DESC_LABEL, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
            NNS_PROPOSAL_SORT_TALLY_TIME_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
            NNS_PROPOSAL_SORT_VOTING_POWER_LABEL,
        },
    },
    source::{
        NnsProposalSource, NnsProposalSourceFuture,
        build_nns_proposal_list_report_with_source as build_nns_proposal_list_report_async,
        build_nns_proposal_report_with_source as build_nns_proposal_report_async,
        nns_proposal_row_from_info,
    },
    text::{nns_proposal_list_report_text, nns_proposal_report_text},
    view::proposal_matches_query,
    wire::{
        NnsGovernanceBallot, NnsNeuronId, NnsProposal, NnsProposalAction, NnsProposalId,
        NnsProposalInfo, NnsProposalTallyWire,
    },
};
use crate::{
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    nns::{
        LiveNnsSource, MAINNET_GOVERNANCE_CANISTER_ID,
        governance::{
            NnsGovernanceError, NnsGovernanceRequest, NnsGovernanceSourceData,
            NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
        },
    },
    runtime::block_on_current_thread,
    subnet_catalog::MAINNET_NETWORK,
};
use candid::Reserved;

mod detail;
mod list;
mod portable;
mod sorts;

#[test]
fn live_proposal_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsGovernanceRequest::replica_query(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = block_on_current_thread(LiveNnsSource.fetch_proposals(
        &request,
        1,
        None,
        NnsProposalStatusFilter::Any,
        NnsProposalRewardStatusFilter::Any,
    ))
    .expect("proposal source runtime")
    .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsProposalError::Governance(NnsGovernanceError::UnsupportedNetwork { network })
            if network == "local"
    ));
}

#[derive(Clone, Debug)]
struct FixtureSource {
    expected_status: Vec<i32>,
    expected_reward_status: Vec<i32>,
    proposals: Vec<NnsProposalInfo>,
    proposal: NnsProposalInfo,
}

impl NnsProposalSource for FixtureSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_eq!(limit, 50);
            assert_eq!(before_proposal_id, Some(200));
            let include_status = status
                .governance_status_code()
                .into_iter()
                .collect::<Vec<_>>();
            let include_reward_status = reward_status
                .governance_reward_status_code()
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(include_status, self.expected_status);
            assert_eq!(include_reward_status, self.expected_reward_status);
            Ok(NnsGovernanceSourceData::new(
                self.proposals
                    .clone()
                    .into_iter()
                    .map(nns_proposal_row_from_info)
                    .collect(),
                fixture_provenance(request),
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            assert_eq!(proposal_id, 101);
            Ok(NnsGovernanceSourceData::new(
                nns_proposal_row_from_info(self.proposal.clone()),
                fixture_provenance(request),
            ))
        })
    }
}

fn build_nns_proposal_list_report_with_source(
    request: &NnsProposalListRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalListReport, NnsProposalError> {
    block_on_current_thread(build_nns_proposal_list_report_async(request, source))
        .expect("proposal list test runtime")
}

fn build_nns_proposal_report_with_source(
    request: &NnsProposalRequest,
    source: &dyn NnsProposalSource,
) -> Result<super::model::NnsProposalReport, NnsProposalError> {
    block_on_current_thread(build_nns_proposal_report_async(request, source))
        .expect("proposal detail test runtime")
}

fn proposal_governance_request(now_unix_secs: u64) -> NnsGovernanceRequest {
    NnsGovernanceRequest::replica_query_from_unix_secs(
        MAINNET_NETWORK,
        DEFAULT_MAINNET_ENDPOINT,
        now_unix_secs,
        "ic-query",
    )
}

fn fixture_provenance(request: &NnsGovernanceRequest) -> NnsGovernanceSourceProvenance {
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        panic!("proposal fixture requires replica-query provenance");
    };
    NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: endpoint.clone(),
        fetched_by: fetched_by.clone(),
    }
}

fn proposal_sort_request(sort: NnsProposalListSort) -> NnsProposalListRequest {
    NnsProposalListRequest {
        governance: proposal_governance_request(1_700_000_000),
        limit: 50,
        before_proposal_id: Some(200),
        status: NnsProposalStatusFilter::Any,
        reward_status: NnsProposalRewardStatusFilter::Any,
        topic: NnsProposalTopicFilter::Any,
        proposer_neuron_id: None,
        query: None,
        sort,
        sort_direction: sort.default_direction(),
        verbose: false,
    }
}

fn proposal_ids(report: &NnsProposalListReport) -> Vec<u64> {
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
            NnsProposalTopic::Governance.code(),
            NnsProposalStatus::Executed.code(),
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
            NnsProposalTopic::Governance.code(),
            NnsProposalStatus::Executed.code(),
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
            NnsProposalTopic::Governance.code(),
            NnsProposalStatus::Executed.code(),
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
