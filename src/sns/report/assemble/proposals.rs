//! Module: sns::report::assemble::proposals
//!
//! Responsibility: assemble SNS proposal list and detail reports.
//! Does not own: proposal fetching, cache loading, view filtering/sorting, or rendering.
//! Boundary: maps resolved proposal rows and provenance into serializable report DTOs.

use super::SnsReportProvenance;
use crate::sns::report::{
    MainnetSns, MainnetSnsList, MainnetSnsProposal, MainnetSnsProposals,
    SNS_PROPOSAL_REPORT_SCHEMA_VERSION, SNS_PROPOSALS_REPORT_SCHEMA_VERSION, SnsProposalReport,
    SnsProposalSortDirection, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsReport,
    SnsProposalsSort,
};

///
/// SnsProposalReportParts
///
/// Inputs needed to assemble one SNS proposal detail report.
///

pub(in crate::sns::report) struct SnsProposalReportParts {
    pub(in crate::sns::report) list: MainnetSnsList,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
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
    pub(in crate::sns::report) list: MainnetSnsList,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
    pub(in crate::sns::report) requested_limit: u32,
    pub(in crate::sns::report) before_proposal_id: Option<u64>,
    pub(in crate::sns::report) status: SnsProposalStatusFilter,
    pub(in crate::sns::report) topic: SnsProposalTopicFilter,
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
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
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
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        before_proposal_id: parts.before_proposal_id,
        status_filter: parts.status.as_str().to_string(),
        topic_filter: parts.topic.as_str().to_string(),
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
