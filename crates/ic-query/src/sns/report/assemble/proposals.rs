//! Module: sns::report::assemble::proposals
//!
//! Responsibility: assemble SNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use super::SnsReportProvenance;
use crate::sns::report::{
    SNS_PROPOSAL_REPORT_SCHEMA_VERSION, SNS_PROPOSALS_REPORT_SCHEMA_VERSION, SnsProposalReport,
    SnsProposalRequest, SnsProposalRow, SnsProposalsReport, SnsProposalsRequest,
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

/// Assemble resolved proposal evidence with the original detail view options.
pub(in crate::sns::report) fn sns_proposal_report_from_parts(
    request: &SnsProposalRequest,
    context: SnsProposalReportContext,
    provenance: SnsReportProvenance,
    proposal: SnsProposalRow,
) -> SnsProposalReport {
    SnsProposalReport {
        schema_version: SNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: context.network,
        sns_wasm_canister_id: context.sns_wasm_canister_id,
        fetched_at: context.fetched_at,
        source_endpoint: context.source_endpoint,
        fetched_by: context.fetched_by,
        id: context.id,
        name: context.name,
        root_canister_id: context.root_canister_id,
        governance_canister_id: context.governance_canister_id,
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        show_ballots: request.show_ballots,
        data_source: provenance.data_source,
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        proposal,
    }
}

/// Assemble resolved proposal evidence with the original list view options.
pub(in crate::sns::report) fn sns_proposals_report_from_parts(
    request: &SnsProposalsRequest,
    context: SnsProposalReportContext,
    provenance: SnsReportProvenance,
    proposals: Vec<SnsProposalRow>,
) -> SnsProposalsReport {
    let proposal_count = proposals.len();
    SnsProposalsReport {
        schema_version: SNS_PROPOSALS_REPORT_SCHEMA_VERSION,
        network: context.network,
        sns_wasm_canister_id: context.sns_wasm_canister_id,
        fetched_at: context.fetched_at,
        source_endpoint: context.source_endpoint,
        fetched_by: context.fetched_by,
        id: context.id,
        name: context.name,
        root_canister_id: context.root_canister_id,
        governance_canister_id: context.governance_canister_id,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        eligibility_filter: request.eligibility.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id.clone(),
        query_filter: request.query.clone(),
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        verbose: request.verbose,
        data_source: provenance.data_source,
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        proposal_count,
        proposals,
    }
}
