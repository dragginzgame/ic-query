//! Module: nns::proposals::report::source
//!
//! Responsibility: build NNS proposal reports from a proposal source.
//! Does not own: CLI parsing, live transport internals, cache IO, or text rendering.
//! Boundary: coordinates source trait calls and converts wire rows to report DTOs.

mod live;

use super::{
    MAINNET_GOVERNANCE_CANISTER_ID, NNS_PROPOSAL_FETCHED_BY, NnsProposalHostError,
    assemble::{
        NnsProposalListReportParts, NnsProposalReportParts, NnsProposalReportProvenance,
        nns_proposal_list_report_from_parts, nns_proposal_report_from_parts,
    },
    enforce_mainnet_network,
    labels::{nns_proposal_status_text, nns_reward_status_text, nns_topic_text, nns_vote_text},
    model::{
        NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalReport,
        NnsProposalRequest, NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter,
        NnsProposalTally,
    },
    view::{
        proposal_matches_proposer, proposal_matches_query, proposal_matches_topic,
        sort_nns_proposal_rows,
    },
    wire::{NnsGovernanceBallot, NnsProposalInfo},
};
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};

pub use live::LiveNnsProposalSource;

pub fn build_nns_proposal_list_report(
    request: &NnsProposalListRequest,
) -> Result<NnsProposalListReport, NnsProposalHostError> {
    build_nns_proposal_list_report_with_source(request, &LiveNnsProposalSource)
}

pub fn build_nns_proposal_report(
    request: &NnsProposalRequest,
) -> Result<NnsProposalReport, NnsProposalHostError> {
    build_nns_proposal_report_with_source(request, &LiveNnsProposalSource)
}

pub fn build_nns_proposal_list_report_with_source(
    request: &NnsProposalListRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalListReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsProposalSourceRequest::new(
        MAINNET_NETWORK,
        &request.source_endpoint,
        &fetched_at,
        NNS_PROPOSAL_FETCHED_BY,
    );
    let mut proposals = source
        .fetch_proposals(
            &fetch_request,
            request.limit,
            request.before_proposal_id,
            request.status,
            request.reward_status,
        )?
        .into_iter()
        .filter(|proposal| proposal_matches_proposer(proposal, request.proposer_neuron_id))
        .filter(|proposal| proposal_matches_topic(proposal, request.topic))
        .filter(|proposal| proposal_matches_query(proposal, request.query.as_deref()))
        .collect::<Vec<_>>();
    sort_nns_proposal_rows(&mut proposals, request.sort, request.sort_direction);
    Ok(nns_proposal_list_report_from_parts(
        NnsProposalListReportParts {
            network: MAINNET_NETWORK.to_string(),
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            fetched_at,
            source_endpoint: request.source_endpoint.clone(),
            fetched_by: NNS_PROPOSAL_FETCHED_BY.to_string(),
            provenance: NnsProposalReportProvenance::live(),
            requested_limit: request.limit,
            before_proposal_id: request.before_proposal_id,
            status: request.status,
            reward_status: request.reward_status,
            topic: request.topic,
            proposer_neuron_id: request.proposer_neuron_id,
            query: request.query.clone(),
            sort: request.sort,
            sort_direction: request.sort_direction,
            verbose: request.verbose,
            proposals,
        },
    ))
}

pub fn build_nns_proposal_report_with_source(
    request: &NnsProposalRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsProposalSourceRequest::new(
        MAINNET_NETWORK,
        &request.source_endpoint,
        &fetched_at,
        NNS_PROPOSAL_FETCHED_BY,
    );
    let proposal = source.fetch_proposal(&fetch_request, request.proposal_id)?;
    Ok(nns_proposal_report_from_parts(NnsProposalReportParts {
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: NNS_PROPOSAL_FETCHED_BY.to_string(),
        provenance: NnsProposalReportProvenance::live(),
        proposal_id: request.proposal_id,
        show_ballots: request.show_ballots,
        verbose: request.verbose,
        proposal,
    }))
}

///
/// NnsProposalSourceRequest
///
/// Source request settings for fetching NNS governance proposal data.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsProposalSourceRequest {
    pub network: String,
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsProposalSourceRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// NnsProposalSource
///
/// Source contract for fetching NNS governance proposal rows.
///

pub trait NnsProposalSource {
    fn fetch_proposals(
        &self,
        request: &NnsProposalSourceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> Result<Vec<NnsProposalRow>, NnsProposalHostError>;

    fn fetch_proposal(
        &self,
        request: &NnsProposalSourceRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalRow, NnsProposalHostError>;
}

pub(in crate::nns::proposals::report) fn nns_proposal_row_from_info(
    info: NnsProposalInfo,
) -> NnsProposalRow {
    let proposal = info.proposal;
    let ballot_count = info.ballots.len();
    let ballots = nns_proposal_ballot_rows(info.ballots);
    NnsProposalRow {
        proposal_id: info.id.map(|id| id.id),
        proposer_neuron_id: info.proposer.map(|id| id.id),
        topic: info.topic,
        topic_text: nns_topic_text(info.topic).to_string(),
        status: info.status,
        status_text: nns_proposal_status_text(info.status).to_string(),
        reward_status: info.reward_status,
        reward_status_text: nns_reward_status_text(info.reward_status).to_string(),
        title: proposal
            .as_ref()
            .and_then(|proposal| proposal.title.clone()),
        summary: proposal
            .as_ref()
            .map_or_else(String::new, |proposal| proposal.summary.clone()),
        url: proposal
            .as_ref()
            .map_or_else(String::new, |proposal| proposal.url.clone()),
        action_text: proposal
            .as_ref()
            .and_then(|proposal| proposal.action.as_ref())
            .map(|action| action.as_str().to_string()),
        reject_cost_e8s: info.reject_cost_e8s,
        proposal_timestamp_seconds: info.proposal_timestamp_seconds,
        proposed_at: format_utc_timestamp_secs(info.proposal_timestamp_seconds),
        deadline_timestamp_seconds: info.deadline_timestamp_seconds,
        deadline_at: info
            .deadline_timestamp_seconds
            .map(format_utc_timestamp_secs),
        decided_timestamp_seconds: info.decided_timestamp_seconds,
        decided_at: nonzero_timestamp_text(info.decided_timestamp_seconds),
        executed_timestamp_seconds: info.executed_timestamp_seconds,
        executed_at: nonzero_timestamp_text(info.executed_timestamp_seconds),
        failed_timestamp_seconds: info.failed_timestamp_seconds,
        failed_at: nonzero_timestamp_text(info.failed_timestamp_seconds),
        reward_event_round: info.reward_event_round,
        total_potential_voting_power: info.total_potential_voting_power,
        latest_tally: info.latest_tally.map(|tally| NnsProposalTally {
            timestamp_seconds: tally.timestamp_seconds,
            yes: tally.yes,
            no: tally.no,
            total: tally.total,
        }),
        ballot_count,
        ballots,
    }
}

fn nns_proposal_ballot_rows(ballots: Vec<(u64, NnsGovernanceBallot)>) -> Vec<NnsProposalBallotRow> {
    let mut rows = ballots
        .into_iter()
        .map(|(neuron_id, ballot)| NnsProposalBallotRow {
            neuron_id,
            vote: ballot.vote,
            vote_text: nns_vote_text(ballot.vote).to_string(),
            voting_power: ballot.voting_power,
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|ballot| ballot.neuron_id);
    rows
}

fn nonzero_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    (timestamp_seconds > 0).then(|| format_utc_timestamp_secs(timestamp_seconds))
}
