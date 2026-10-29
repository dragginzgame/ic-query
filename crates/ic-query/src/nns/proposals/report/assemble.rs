//! Module: nns::proposals::report::assemble
//!
//! Responsibility: assemble NNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use crate::{
    nns::governance::NnsGovernanceReportContext,
    nns::proposals::report::{
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        model::{NnsProposalListReport, NnsProposalListRequest, NnsProposalReport, NnsProposalRow},
    },
    report::{ReportDataSource, ReportResultScope},
};
#[cfg(feature = "nns-host")]
use std::path::Path;

///
/// NnsProposalReportProvenance
///
/// Shared source metadata attached to NNS proposal reports.
///

pub(in crate::nns::proposals::report) struct NnsProposalReportProvenance {
    pub(in crate::nns::proposals::report) data_source: ReportDataSource,
    pub(in crate::nns::proposals::report) cache_path: Option<String>,
    pub(in crate::nns::proposals::report) cache_complete: Option<bool>,
    pub(in crate::nns::proposals::report) result_scope: ReportResultScope,
}

impl NnsProposalReportProvenance {
    /// Build provenance for live NNS governance reports.
    pub(in crate::nns::proposals::report) const fn live() -> Self {
        Self {
            data_source: ReportDataSource::Live,
            cache_path: None,
            cache_complete: None,
            result_scope: ReportResultScope::BoundedLive,
        }
    }

    /// Build provenance for complete-cache NNS governance reports.
    #[cfg(feature = "nns-host")]
    pub(in crate::nns::proposals::report) fn cache(cache_path: &Path) -> Self {
        Self {
            data_source: ReportDataSource::Cache,
            cache_path: Some(cache_path.display().to_string()),
            cache_complete: Some(true),
            result_scope: ReportResultScope::CompleteCache,
        }
    }
}

///
/// NnsProposalReportParts
///
/// Inputs needed to assemble one NNS proposal detail report.
///

pub(in crate::nns::proposals::report) struct NnsProposalReportParts {
    pub(in crate::nns::proposals::report) context: NnsGovernanceReportContext,
    pub(in crate::nns::proposals::report) proposal_id: u64,
    pub(in crate::nns::proposals::report) show_ballots: bool,
    pub(in crate::nns::proposals::report) verbose: bool,
    pub(in crate::nns::proposals::report) provenance: NnsProposalReportProvenance,
    pub(in crate::nns::proposals::report) proposal: NnsProposalRow,
}

/// Assemble an NNS proposal list from request settings, resolved rows and provenance.
pub(in crate::nns::proposals::report) fn nns_proposal_list_report_from_rows(
    request: &NnsProposalListRequest,
    context: NnsGovernanceReportContext,
    provenance: NnsProposalReportProvenance,
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
        result_scope: provenance.result_scope,
        verbose: request.verbose,
        proposal_count: proposals.len(),
        proposals,
    }
}

/// Assemble an NNS proposal detail report from resolved proposal parts.
pub(in crate::nns::proposals::report) fn nns_proposal_report_from_parts(
    parts: NnsProposalReportParts,
) -> NnsProposalReport {
    NnsProposalReport {
        context: NnsGovernanceReportContext {
            schema_version: NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
            ..parts.context
        },
        data_source: parts.provenance.data_source,
        cache_path: parts.provenance.cache_path,
        cache_complete: parts.provenance.cache_complete,
        proposal_id: parts.proposal_id,
        show_ballots: parts.show_ballots,
        verbose: parts.verbose,
        proposal: parts.proposal,
    }
}
