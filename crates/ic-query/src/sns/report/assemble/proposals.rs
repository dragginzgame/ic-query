//! Module: sns::report::assemble::proposals
//!
//! Responsibility: assemble SNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use super::SnsReportProvenance;
use crate::sns::report::{
    MainnetSnsProposal, MainnetSnsProposals, SNS_PROPOSAL_REPORT_SCHEMA_VERSION,
    SNS_PROPOSALS_REPORT_SCHEMA_VERSION, SnsProposalEligibilityFilter, SnsProposalReport,
    SnsProposalSortDirection, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsReport,
    SnsProposalsSort,
};

///
/// SnsProposalReportContext
///
/// Validated identity and acquisition metadata used by proposal reports.
///

pub(in crate::sns::report) struct SnsProposalReportContext {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

///
/// SnsProposalReportParts
///
/// Inputs needed to assemble one SNS proposal detail report.
///

pub(in crate::sns::report) struct SnsProposalReportParts {
    pub(in crate::sns::report) context: SnsProposalReportContext,
    pub(in crate::sns::report) proposal_id: u64,
    pub(in crate::sns::report) verbose: bool,
    pub(in crate::sns::report) show_ballots: bool,
    pub(in crate::sns::report) provenance: SnsReportProvenance,
    pub(in crate::sns::report) proposal: MainnetSnsProposal,
}

///
/// SnsProposalsReportParts
///
/// Inputs needed to assemble one SNS proposal list report.
///

pub(in crate::sns::report) struct SnsProposalsReportParts {
    pub(in crate::sns::report) context: SnsProposalReportContext,
    pub(in crate::sns::report) requested_limit: u32,
    pub(in crate::sns::report) before_proposal_id: Option<u64>,
    pub(in crate::sns::report) status: SnsProposalStatusFilter,
    pub(in crate::sns::report) topic: SnsProposalTopicFilter,
    pub(in crate::sns::report) eligibility: SnsProposalEligibilityFilter,
    pub(in crate::sns::report) proposer_neuron_id: Option<String>,
    pub(in crate::sns::report) query: Option<String>,
    pub(in crate::sns::report) sort: SnsProposalsSort,
    pub(in crate::sns::report) sort_direction: SnsProposalSortDirection,
    pub(in crate::sns::report) verbose: bool,
    pub(in crate::sns::report) provenance: SnsReportProvenance,
    pub(in crate::sns::report) proposals: MainnetSnsProposals,
}

/// Assemble an SNS proposal detail report from resolved proposal parts.
pub(in crate::sns::report) fn sns_proposal_report_from_parts(
    parts: SnsProposalReportParts,
) -> SnsProposalReport {
    SnsProposalReport {
        schema_version: SNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: parts.context.network,
        sns_wasm_canister_id: parts.context.sns_wasm_canister_id,
        fetched_at: parts.context.fetched_at,
        source_endpoint: parts.context.source_endpoint,
        fetched_by: parts.context.fetched_by,
        id: parts.context.id,
        name: parts.context.name,
        root_canister_id: parts.context.root_canister_id,
        governance_canister_id: parts.context.governance_canister_id,
        proposal_id: parts.proposal_id,
        verbose: parts.verbose,
        show_ballots: parts.show_ballots,
        data_source: parts.provenance.data_source,
        cache_path: parts.provenance.cache_path,
        cache_complete: parts.provenance.cache_complete,
        proposal: parts.proposal.proposal,
    }
}

/// Assemble an SNS proposal list report from resolved proposal parts.
pub(in crate::sns::report) fn sns_proposals_report_from_parts(
    parts: SnsProposalsReportParts,
) -> SnsProposalsReport {
    let proposal_count = parts.proposals.proposals.len();
    SnsProposalsReport {
        schema_version: SNS_PROPOSALS_REPORT_SCHEMA_VERSION,
        network: parts.context.network,
        sns_wasm_canister_id: parts.context.sns_wasm_canister_id,
        fetched_at: parts.context.fetched_at,
        source_endpoint: parts.context.source_endpoint,
        fetched_by: parts.context.fetched_by,
        id: parts.context.id,
        name: parts.context.name,
        root_canister_id: parts.context.root_canister_id,
        governance_canister_id: parts.context.governance_canister_id,
        requested_limit: parts.requested_limit,
        before_proposal_id: parts.before_proposal_id,
        status_filter: parts.status.as_str().to_string(),
        topic_filter: parts.topic.as_str().to_string(),
        eligibility_filter: parts.eligibility.as_str().to_string(),
        proposer_filter: parts.proposer_neuron_id,
        query_filter: parts.query,
        sort: parts.sort.as_str().to_string(),
        sort_direction: parts.sort.direction_label(parts.sort_direction).to_string(),
        verbose: parts.verbose,
        data_source: parts.provenance.data_source,
        cache_path: parts.provenance.cache_path,
        cache_complete: parts.provenance.cache_complete,
        proposal_count,
        proposals: parts.proposals.proposals,
    }
}
