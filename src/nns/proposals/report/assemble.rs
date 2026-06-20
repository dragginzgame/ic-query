//! Module: nns::proposals::report::assemble
//!
//! Responsibility: assemble NNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use crate::nns::proposals::report::{
    NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
    model::{
        NnsProposalListReport, NnsProposalListSort, NnsProposalReport,
        NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalSortDirection,
        NnsProposalStatusFilter, NnsProposalTopicFilter,
    },
};
use std::path::Path;

///
/// NnsProposalReportProvenance
///
/// Shared source metadata attached to NNS proposal reports.
///

pub(in crate::nns::proposals::report) struct NnsProposalReportProvenance {
    pub(in crate::nns::proposals::report) data_source: String,
    pub(in crate::nns::proposals::report) cache_path: Option<String>,
    pub(in crate::nns::proposals::report) cache_complete: Option<bool>,
}

impl NnsProposalReportProvenance {
    /// Build provenance for live NNS governance reports.
    pub(in crate::nns::proposals::report) fn live() -> Self {
        Self {
            data_source: "live".to_string(),
            cache_path: None,
            cache_complete: None,
        }
    }

    /// Build provenance for complete-cache NNS governance reports.
    pub(in crate::nns::proposals::report) fn cache(
        cache_path: &Path,
        cache_complete: bool,
    ) -> Self {
        Self {
            data_source: "cache".to_string(),
            cache_path: Some(cache_path.display().to_string()),
            cache_complete: Some(cache_complete),
        }
    }
}

///
/// NnsProposalListReportParts
///
/// Inputs needed to assemble one NNS proposal list report.
///

pub(in crate::nns::proposals::report) struct NnsProposalListReportParts {
    pub(in crate::nns::proposals::report) network: String,
    pub(in crate::nns::proposals::report) governance_canister_id: String,
    pub(in crate::nns::proposals::report) fetched_at: String,
    pub(in crate::nns::proposals::report) source_endpoint: String,
    pub(in crate::nns::proposals::report) fetched_by: String,
    pub(in crate::nns::proposals::report) requested_limit: u32,
    pub(in crate::nns::proposals::report) before_proposal_id: Option<u64>,
    pub(in crate::nns::proposals::report) status: NnsProposalStatusFilter,
    pub(in crate::nns::proposals::report) reward_status: NnsProposalRewardStatusFilter,
    pub(in crate::nns::proposals::report) topic: NnsProposalTopicFilter,
    pub(in crate::nns::proposals::report) sort: NnsProposalListSort,
    pub(in crate::nns::proposals::report) sort_direction: NnsProposalSortDirection,
    pub(in crate::nns::proposals::report) verbose: bool,
    pub(in crate::nns::proposals::report) provenance: NnsProposalReportProvenance,
    pub(in crate::nns::proposals::report) proposals: Vec<NnsProposalRow>,
}

///
/// NnsProposalReportParts
///
/// Inputs needed to assemble one NNS proposal detail report.
///

pub(in crate::nns::proposals::report) struct NnsProposalReportParts {
    pub(in crate::nns::proposals::report) network: String,
    pub(in crate::nns::proposals::report) governance_canister_id: String,
    pub(in crate::nns::proposals::report) fetched_at: String,
    pub(in crate::nns::proposals::report) source_endpoint: String,
    pub(in crate::nns::proposals::report) fetched_by: String,
    pub(in crate::nns::proposals::report) proposal_id: u64,
    pub(in crate::nns::proposals::report) show_ballots: bool,
    pub(in crate::nns::proposals::report) verbose: bool,
    pub(in crate::nns::proposals::report) provenance: NnsProposalReportProvenance,
    pub(in crate::nns::proposals::report) proposal: NnsProposalRow,
}

/// Assemble an NNS proposal list report from resolved proposal parts.
pub(in crate::nns::proposals::report) fn nns_proposal_list_report_from_parts(
    parts: NnsProposalListReportParts,
) -> NnsProposalListReport {
    let proposal_count = parts.proposals.len();
    NnsProposalListReport {
        schema_version: NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION,
        network: parts.network,
        governance_canister_id: parts.governance_canister_id,
        fetched_at: parts.fetched_at,
        source_endpoint: parts.source_endpoint,
        fetched_by: parts.fetched_by,
        data_source: parts.provenance.data_source,
        cache_path: parts.provenance.cache_path,
        cache_complete: parts.provenance.cache_complete,
        requested_limit: parts.requested_limit,
        before_proposal_id: parts.before_proposal_id,
        status_filter: parts.status.as_str().to_string(),
        reward_status_filter: parts.reward_status.as_str().to_string(),
        topic_filter: parts.topic.as_str().to_string(),
        sort: parts.sort.as_str().to_string(),
        sort_direction: parts.sort.direction_label(parts.sort_direction).to_string(),
        verbose: parts.verbose,
        proposal_count,
        proposals: parts.proposals,
    }
}

/// Assemble an NNS proposal detail report from resolved proposal parts.
pub(in crate::nns::proposals::report) fn nns_proposal_report_from_parts(
    parts: NnsProposalReportParts,
) -> NnsProposalReport {
    NnsProposalReport {
        schema_version: NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: parts.network,
        governance_canister_id: parts.governance_canister_id,
        fetched_at: parts.fetched_at,
        source_endpoint: parts.source_endpoint,
        fetched_by: parts.fetched_by,
        data_source: parts.provenance.data_source,
        cache_path: parts.provenance.cache_path,
        cache_complete: parts.provenance.cache_complete,
        proposal_id: parts.proposal_id,
        show_ballots: parts.show_ballots,
        verbose: parts.verbose,
        proposal: parts.proposal,
    }
}
