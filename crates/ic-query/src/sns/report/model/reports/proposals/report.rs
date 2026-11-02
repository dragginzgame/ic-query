//! Module: sns::report::model::reports::proposals::report
//!
//! Responsibility: define SNS proposal list and detail report DTOs.
//! Does not own: live governance calls, row conversion, cache reads, or rendering.
//! Boundary: preserves report-level fields for text and JSON output.

use super::row::SnsProposalRow;
use crate::report::ReportProvenance;
use crate::sns::report::SnsReportContext;
use serde::Serialize;

///
/// SnsProposalReport
///
/// Serializable report for one SNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalReport {
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    pub proposal_id: u64,
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: ReportProvenance,
    pub proposal: SnsProposalRow,
}

///
/// SnsProposalsReport
///
/// Serializable report for a bounded SNS governance proposal listing.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsReport {
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    pub requested_limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status_filter: String,
    pub topic_filter: String,
    pub eligibility_filter: String,
    pub proposer_filter: Option<String>,
    pub query_filter: Option<String>,
    pub sort: String,
    pub sort_direction: String,
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: ReportProvenance,
    pub proposal_count: usize,
    pub proposals: Vec<SnsProposalRow>,
}
