//! Module: nns::proposals::report::source
//!
//! Responsibility: fetch and assemble live NNS governance proposal reports.
//! Does not own: CLI parsing, Candid wire definitions, or text rendering.
//! Boundary: executes mainnet governance queries and converts wire rows to report DTOs.

use super::{
    MAINNET_GOVERNANCE_CANISTER_ID, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
    NNS_PROPOSALS_REPORT_SCHEMA_VERSION, NnsProposalHostError, enforce_mainnet_network,
    model::{
        NNS_PROPOSAL_REWARD_STATUS_ACCEPT_VOTES_LABEL, NNS_PROPOSAL_REWARD_STATUS_INELIGIBLE_LABEL,
        NNS_PROPOSAL_REWARD_STATUS_READY_TO_SETTLE_LABEL, NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
        NNS_PROPOSAL_STATUS_ADOPTED_LABEL, NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
        NNS_PROPOSAL_STATUS_FAILED_LABEL, NNS_PROPOSAL_STATUS_OPEN_LABEL,
        NNS_PROPOSAL_STATUS_REJECTED_LABEL, NNS_PROPOSAL_STATUS_UNSPECIFIED_LABEL,
        NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_LABEL,
        NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_LABEL, NNS_PROPOSAL_TOPIC_KYC_LABEL,
        NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_LABEL, NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_NODE_ADMIN_LABEL, NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_LABEL,
        NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL,
        NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_LABEL,
        NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_LABEL, NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_LABEL,
        NnsProposalReport, NnsProposalRequest, NnsProposalRow, NnsProposalTally,
        NnsProposalsReport, NnsProposalsRequest,
    },
    view::{proposal_matches_topic, sort_nns_proposal_rows},
    wire::{
        NnsListProposalInfoRequest, NnsListProposalInfoResponse, NnsProposalId, NnsProposalInfo,
    },
};
use crate::{
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::{CandidType, Deserialize, Principal};
use ic_agent::Agent;

pub(in crate::nns::proposals) fn build_nns_proposals_report(
    request: &NnsProposalsRequest,
) -> Result<NnsProposalsReport, NnsProposalHostError> {
    build_nns_proposals_report_with_source(request, &LiveNnsProposalSource)
}

pub(in crate::nns::proposals) fn build_nns_proposal_report(
    request: &NnsProposalRequest,
) -> Result<NnsProposalReport, NnsProposalHostError> {
    build_nns_proposal_report_with_source(request, &LiveNnsProposalSource)
}

pub(in crate::nns::proposals::report) fn build_nns_proposals_report_with_source(
    request: &NnsProposalsRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalsReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsProposalFetchRequest::new(&request.source_endpoint, &fetched_at);
    let include_status = request
        .status
        .governance_status_code()
        .into_iter()
        .collect::<Vec<_>>();
    let proposal_infos = source.fetch_proposals(
        &fetch_request,
        request.limit,
        request.before_proposal_id,
        &include_status,
    )?;
    let mut proposals = proposal_infos
        .into_iter()
        .map(nns_proposal_row_from_info)
        .filter(|proposal| proposal_matches_topic(proposal, request.topic))
        .collect::<Vec<_>>();
    sort_nns_proposal_rows(&mut proposals, request.sort, request.sort_direction);
    Ok(NnsProposalsReport {
        schema_version: NNS_PROPOSALS_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        verbose: request.verbose,
        proposal_count: proposals.len(),
        proposals,
    })
}

pub(in crate::nns::proposals::report) fn build_nns_proposal_report_with_source(
    request: &NnsProposalRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsProposalFetchRequest::new(&request.source_endpoint, &fetched_at);
    let proposal_info = source.fetch_proposal(&fetch_request, request.proposal_id)?;
    Ok(NnsProposalReport {
        schema_version: NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        proposal_id: request.proposal_id,
        proposal: nns_proposal_row_from_info(proposal_info),
    })
}

///
/// NnsProposalFetchRequest
///
/// Live source request metadata for NNS governance proposal calls.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsProposalFetchRequest {
    pub(in crate::nns::proposals::report) endpoint: String,
    pub(in crate::nns::proposals::report) fetched_at: String,
    pub(in crate::nns::proposals::report) fetched_by: String,
}

impl NnsProposalFetchRequest {
    fn new(endpoint: &str, fetched_at: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            fetched_at: fetched_at.to_string(),
            fetched_by: "ic-query".to_string(),
        }
    }
}

///
/// NnsProposalSource
///
/// Source trait for NNS governance proposal list and detail calls.
///

pub(in crate::nns::proposals::report) trait NnsProposalSource {
    fn fetch_proposals(
        &self,
        request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError>;

    fn fetch_proposal(
        &self,
        request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError>;
}

///
/// LiveNnsProposalSource
///
/// Live source backed by the mainnet NNS governance canister.
///

pub(in crate::nns::proposals::report) struct LiveNnsProposalSource;

impl NnsProposalSource for LiveNnsProposalSource {
    fn fetch_proposals(
        &self,
        request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        block_on_current_thread(fetch_nns_proposals_async(
            request,
            limit,
            before_proposal_id,
            include_status,
        ))
        .map_err(NnsProposalHostError::Runtime)?
    }

    fn fetch_proposal(
        &self,
        request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError> {
        block_on_current_thread(fetch_nns_proposal_async(request, proposal_id))
            .map_err(NnsProposalHostError::Runtime)?
    }
}

async fn fetch_nns_proposals_async(
    request: &NnsProposalFetchRequest,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
    let agent = nns_agent(&request.endpoint)?;
    let governance_canister = governance_canister()?;
    let response: NnsListProposalInfoResponse = query_canister(
        &agent,
        &governance_canister,
        "list_proposals",
        "ListProposalInfoRequest",
        "ListProposalInfoResponse",
        &NnsListProposalInfoRequest {
            include_reward_status: Vec::new(),
            omit_large_fields: Some(false),
            before_proposal: before_proposal_id.map(|id| NnsProposalId { id }),
            limit,
            exclude_topic: Vec::new(),
            include_all_manage_neuron_proposals: Some(true),
            include_status: include_status.to_vec(),
            return_self_describing_action: Some(false),
        },
    )
    .await?;
    Ok(response.proposal_info)
}

async fn fetch_nns_proposal_async(
    request: &NnsProposalFetchRequest,
    proposal_id: u64,
) -> Result<NnsProposalInfo, NnsProposalHostError> {
    let agent = nns_agent(&request.endpoint)?;
    let governance_canister = governance_canister()?;
    let proposal: Option<NnsProposalInfo> = query_canister(
        &agent,
        &governance_canister,
        "get_proposal_info",
        "ProposalId",
        "ProposalInfo",
        &proposal_id,
    )
    .await?;
    proposal.ok_or(NnsProposalHostError::ProposalNotFound { proposal_id })
}

async fn query_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, NnsProposalHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| NnsProposalHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| NnsProposalHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| NnsProposalHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
    })
}

fn nns_agent(endpoint: &str) -> Result<Agent, NnsProposalHostError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| NnsProposalHostError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

fn governance_canister() -> Result<Principal, NnsProposalHostError> {
    Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|err| {
        NnsProposalHostError::CandidDecode {
            message: "governance_canister_id",
            reason: err.to_string(),
        }
    })
}

fn nns_proposal_row_from_info(info: NnsProposalInfo) -> NnsProposalRow {
    let proposal = info.proposal;
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
        ballot_count: info.ballots.len(),
    }
}

fn nonzero_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    (timestamp_seconds > 0).then(|| format_utc_timestamp_secs(timestamp_seconds))
}

const fn nns_proposal_status_text(status: i32) -> &'static str {
    match status {
        1 => NNS_PROPOSAL_STATUS_OPEN_LABEL,
        2 => NNS_PROPOSAL_STATUS_REJECTED_LABEL,
        3 => NNS_PROPOSAL_STATUS_ADOPTED_LABEL,
        4 => NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
        5 => NNS_PROPOSAL_STATUS_FAILED_LABEL,
        _ => NNS_PROPOSAL_STATUS_UNSPECIFIED_LABEL,
    }
}

const fn nns_reward_status_text(status: i32) -> &'static str {
    match status {
        1 => NNS_PROPOSAL_REWARD_STATUS_ACCEPT_VOTES_LABEL,
        2 => NNS_PROPOSAL_REWARD_STATUS_READY_TO_SETTLE_LABEL,
        3 => NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
        4 => NNS_PROPOSAL_REWARD_STATUS_INELIGIBLE_LABEL,
        _ => NNS_PROPOSAL_STATUS_UNSPECIFIED_LABEL,
    }
}

const fn nns_topic_text(topic: i32) -> &'static str {
    match topic {
        1 => NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_LABEL,
        2 => NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_LABEL,
        3 => NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_LABEL,
        4 => NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        5 => NNS_PROPOSAL_TOPIC_NODE_ADMIN_LABEL,
        6 => NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_LABEL,
        7 => NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_LABEL,
        8 => NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_LABEL,
        9 => NNS_PROPOSAL_TOPIC_KYC_LABEL,
        10 => NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_LABEL,
        12 => NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_LABEL,
        13 => NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_LABEL,
        14 => NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_LABEL,
        15 => NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_LABEL,
        16 => NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_LABEL,
        17 => NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_LABEL,
        18 => NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL,
        _ => NNS_PROPOSAL_STATUS_UNSPECIFIED_LABEL,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL, NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
        NNS_PROPOSAL_STATUS_OPEN_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL, nns_proposal_status_text,
        nns_reward_status_text, nns_topic_text,
    };

    #[test]
    fn nns_proposal_labels_cover_common_values() {
        assert_eq!(nns_proposal_status_text(1), NNS_PROPOSAL_STATUS_OPEN_LABEL);
        assert_eq!(
            nns_proposal_status_text(4),
            NNS_PROPOSAL_STATUS_EXECUTED_LABEL
        );
        assert_eq!(
            nns_reward_status_text(3),
            NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL
        );
        assert_eq!(nns_topic_text(4), NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL);
        assert_eq!(
            nns_topic_text(18),
            NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL
        );
    }
}
