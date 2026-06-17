//! Module: sns::report::proposals_cache::reports::cached_report::filter
//!
//! Responsibility: apply cache-backed SNS proposal list view filters.
//! Does not own: cache loading, report projection, or live API filtering.
//! Boundary: keeps local proposal view filtering explicit and testable.

use crate::sns::report::{SnsProposalRow, SnsProposalStatusFilter};

pub(super) fn proposal_matches_before(
    proposal: &SnsProposalRow,
    before_proposal_id: Option<u64>,
) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

pub(super) fn proposal_matches_status(
    proposal: &SnsProposalRow,
    status: SnsProposalStatusFilter,
) -> bool {
    match status {
        SnsProposalStatusFilter::Any => true,
        SnsProposalStatusFilter::Open => proposal.decision_state == "open",
        SnsProposalStatusFilter::Executed => proposal.decision_state == "executed",
        SnsProposalStatusFilter::Failed => proposal.decision_state == "failed",
        SnsProposalStatusFilter::Rejected | SnsProposalStatusFilter::Adopted => false,
    }
}
