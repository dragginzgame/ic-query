//! Module: nns::proposals::report::assemble
//!
//! Responsibility: assemble NNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use crate::{
    nns::governance::NnsGovernanceReportContext,
    nns::proposals::report::{
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        model::{
            NnsProposalListReport, NnsProposalListRequest, NnsProposalReport, NnsProposalRequest,
            NnsProposalRow,
        },
    },
    report::{ReportDataSource, ReportProvenance, ReportResultScope},
};
/// Assemble an NNS proposal list from request settings, resolved rows and provenance.
pub(in crate::nns::proposals::report) fn nns_proposal_list_report_from_rows(
    request: &NnsProposalListRequest,
    context: NnsGovernanceReportContext,
    provenance: ReportProvenance,
    proposals: Vec<NnsProposalRow>,
) -> NnsProposalListReport {
    NnsProposalListReport {
        context: NnsGovernanceReportContext {
            schema_version: NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION,
            ..context
        },
        data_source: provenance.data_source,
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        reward_status_filter: request.reward_status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id,
        query_filter: request.query.clone(),
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        result_scope: match provenance.data_source {
            ReportDataSource::Live => ReportResultScope::BoundedLive,
            ReportDataSource::Cache => ReportResultScope::CompleteCache,
        },
        verbose: request.verbose,
        proposal_count: proposals.len(),
        proposals,
    }
}

/// Assemble an NNS proposal detail from its request, resolved row and provenance.
pub(in crate::nns::proposals::report) fn nns_proposal_report_from_row(
    request: &NnsProposalRequest,
    context: NnsGovernanceReportContext,
    provenance: ReportProvenance,
    proposal: NnsProposalRow,
) -> NnsProposalReport {
    NnsProposalReport {
        context: NnsGovernanceReportContext {
            schema_version: NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
            ..context
        },
        data_source: provenance.data_source,
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        proposal_id: request.proposal_id,
        show_ballots: request.show_ballots,
        verbose: request.verbose,
        proposal,
    }
}
