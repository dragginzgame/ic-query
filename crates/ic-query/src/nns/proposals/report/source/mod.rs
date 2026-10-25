//! Module: nns::proposals::report::source
//!
//! Responsibility: build NNS proposal reports from a portable async source.
//! Does not own: CLI parsing, cache IO, transport internals, or text rendering.
//! Boundary: native, canister, and custom sources converge before report assembly.

#[cfg(all(feature = "canister", target_arch = "wasm32"))]
mod canister;
#[cfg(feature = "nns-host")]
mod host;

use super::{
    NNS_PROPOSAL_MAX_PAGE_SIZE, NnsProposalError,
    assemble::{
        NnsProposalReportParts, NnsProposalReportProvenance, nns_proposal_list_report_from_rows,
        nns_proposal_report_from_parts,
    },
    model::{
        NnsProposalListReport, NnsProposalListRequest, NnsProposalReport, NnsProposalRequest,
        NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter,
    },
    view::project_nns_proposal_rows,
};
#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
use super::{
    model::{NnsProposalBallotRow, NnsProposalTally},
    wire::{NnsGovernanceBallot, NnsListProposalInfoRequest, NnsProposalId, NnsProposalInfo},
};
#[cfg(feature = "nns-host")]
use crate::nns::LiveNnsSource;
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::{
        NnsGovernanceReportContext, NnsGovernanceRequest, NnsGovernanceSourceData,
        NnsGovernanceSourceProvenance, validate_governance_request, validate_source_provenance,
    },
};
use std::{collections::HashSet, future::Future, pin::Pin};

/// Build one bounded proposal list through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_proposal_list_report(
    request: &NnsProposalListRequest,
) -> Result<NnsProposalListReport, super::NnsProposalHostError> {
    Ok(build_nns_proposal_list_report_with_source(request, &LiveNnsSource::default()).await?)
}

/// Build one exact proposal detail through the native replica adapter.
#[cfg(feature = "nns-host")]
pub async fn build_nns_proposal_report(
    request: &NnsProposalRequest,
) -> Result<NnsProposalReport, super::NnsProposalHostError> {
    Ok(build_nns_proposal_report_with_source(request, &LiveNnsSource::default()).await?)
}

/// Build one bounded proposal list from a caller-owned async source.
pub async fn build_nns_proposal_list_report_with_source(
    request: &NnsProposalListRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalListReport, NnsProposalError> {
    validate_list_request(request)?;
    let data = source
        .fetch_proposals(
            &request.governance,
            request.limit,
            request.before_proposal_id,
            request.status,
            request.reward_status,
        )
        .await?;
    validate_source_provenance(&request.governance.source, &data.provenance)?;
    validate_proposal_page(&data.value, request.limit, request.before_proposal_id)?;

    let proposals = project_nns_proposal_rows(data.value, request);
    Ok(nns_proposal_list_report_from_rows(
        request,
        report_context(&request.governance, data.provenance),
        NnsProposalReportProvenance::live(),
        proposals,
    ))
}

/// Build one exact proposal detail from a caller-owned async source.
pub async fn build_nns_proposal_report_with_source(
    request: &NnsProposalRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalReport, NnsProposalError> {
    validate_governance_request(&request.governance)?;
    let data = source
        .fetch_proposal(&request.governance, request.proposal_id)
        .await?;
    validate_source_provenance(&request.governance.source, &data.provenance)?;
    if data.value.proposal_id != Some(request.proposal_id) {
        return Err(NnsProposalError::ProposalIdMismatch {
            expected: request.proposal_id,
            actual: data.value.proposal_id,
        });
    }
    Ok(nns_proposal_report_from_parts(NnsProposalReportParts {
        context: report_context(&request.governance, data.provenance),
        provenance: NnsProposalReportProvenance::live(),
        proposal_id: request.proposal_id,
        show_ballots: request.show_ballots,
        verbose: request.verbose,
        proposal: data.value,
    }))
}

///
/// NnsProposalSourceFuture
///
/// Boxed caller-runtime future returned by a proposal source.
///

pub type NnsProposalSourceFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<NnsGovernanceSourceData<T>, NnsProposalError>> + Send + 'a>>;

///
/// NnsProposalSource
///
/// Portable async capability for bounded proposal list and exact detail calls.
///

pub trait NnsProposalSource: Send + Sync {
    /// Fetch at most one bounded page of proposal rows.
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>>;

    /// Fetch one exact proposal row.
    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow>;
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
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

        status: info.status,

        reward_status: info.reward_status,

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
        proposed_at: ic_host_text::timestamp::format_utc_timestamp_secs(
            info.proposal_timestamp_seconds,
        ),
        deadline_timestamp_seconds: info.deadline_timestamp_seconds,
        deadline_at: info
            .deadline_timestamp_seconds
            .map(ic_host_text::timestamp::format_utc_timestamp_secs),
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

fn validate_list_request(request: &NnsProposalListRequest) -> Result<(), NnsProposalError> {
    validate_governance_request(&request.governance)?;
    validate_proposal_page_size(request.limit)
}

pub(super) fn validate_proposal_page_size(limit: u32) -> Result<(), NnsProposalError> {
    if (1..=NNS_PROPOSAL_MAX_PAGE_SIZE).contains(&limit) {
        Ok(())
    } else {
        Err(NnsProposalError::InvalidLimit {
            limit,
            maximum: NNS_PROPOSAL_MAX_PAGE_SIZE,
        })
    }
}

fn validate_proposal_page(
    proposals: &[NnsProposalRow],
    requested: u32,
    before_proposal_id: Option<u64>,
) -> Result<(), NnsProposalError> {
    if proposals.len() > requested as usize {
        return Err(NnsProposalError::PageTooLarge {
            actual: proposals.len(),
            requested,
        });
    }
    let mut proposal_ids = HashSet::with_capacity(proposals.len());
    for proposal in proposals {
        let proposal_id = proposal
            .proposal_id
            .ok_or(NnsProposalError::MissingProposalIdInPage)?;
        if proposal_id == 0 {
            return Err(NnsProposalError::InvalidProposalIdInPage);
        }
        if !proposal_ids.insert(proposal_id) {
            return Err(NnsProposalError::DuplicateProposalId { proposal_id });
        }
        if let Some(before_proposal_id) = before_proposal_id
            && proposal_id >= before_proposal_id
        {
            return Err(NnsProposalError::ProposalCursorMismatch {
                proposal_id,
                before_proposal_id,
            });
        }
    }
    Ok(())
}

fn report_context(
    request: &NnsGovernanceRequest,
    source: NnsGovernanceSourceProvenance,
) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: 1,
        network: request.network.clone(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source,
    }
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
fn nns_proposal_ballot_rows(ballots: Vec<(u64, NnsGovernanceBallot)>) -> Vec<NnsProposalBallotRow> {
    let mut rows = ballots
        .into_iter()
        .map(|(neuron_id, ballot)| NnsProposalBallotRow {
            neuron_id,
            vote: ballot.vote,

            voting_power: ballot.voting_power,
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|ballot| ballot.neuron_id);
    rows
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
fn nonzero_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    (timestamp_seconds > 0)
        .then(|| ic_host_text::timestamp::format_utc_timestamp_secs(timestamp_seconds))
}

#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
fn proposal_page_request(
    limit: u32,
    before_proposal_id: Option<u64>,
    status: NnsProposalStatusFilter,
    reward_status: NnsProposalRewardStatusFilter,
) -> NnsListProposalInfoRequest {
    NnsListProposalInfoRequest {
        include_reward_status: reward_status
            .governance_reward_status_code()
            .into_iter()
            .collect(),
        omit_large_fields: Some(false),
        before_proposal: before_proposal_id.map(|id| NnsProposalId { id }),
        limit,
        exclude_topic: Vec::new(),
        include_all_manage_neuron_proposals: Some(true),
        include_status: status.governance_status_code().into_iter().collect(),
        return_self_describing_action: Some(false),
    }
}
