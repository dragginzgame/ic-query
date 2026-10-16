//! Module: sns::report::source::model::proposals
//!
//! Responsibility: validate source-layer SNS proposal rows.
//! Does not own: governance transport, proposal conversion, or rendering.
//! Boundary: checks row identities, derived fields, limits and proposal-page cursors.

use super::validation::SnsSourceValidator;
use crate::sns::report::{SnsHostError, SnsProposalAction, SnsProposalRow, SnsProposalVote};
use std::collections::HashSet;

const PROPOSAL_CAPABILITY: &str = "SNS proposal";
const PROPOSALS_CAPABILITY: &str = "SNS proposals";
const PROPOSAL_PAGE_CAPABILITY: &str = "SNS proposal page";

/// Validate one bounded proposal result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposals(
    proposals: &[SnsProposalRow],
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    validate_sns_proposal_source_rows(proposals, requested_limit, PROPOSALS_CAPABILITY)
}

/// Validate one proposal page returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposal_page(
    page: &[SnsProposalRow],
    requested_limit: u32,
    before_proposal_id: Option<u64>,
) -> Result<(), SnsHostError> {
    validate_sns_proposal_source_rows(page, requested_limit, PROPOSAL_PAGE_CAPABILITY)?;
    if let Some(before) = before_proposal_id
        && page.iter().any(|row| row.proposal_id >= before)
    {
        return Err(SnsSourceValidator::new(PROPOSAL_PAGE_CAPABILITY)
            .invalid("proposal page violates its exclusive before cursor".to_string()));
    }
    Ok(())
}

/// Validate one exact proposal returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposal(
    proposal: &SnsProposalRow,
    requested_proposal_id: u64,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new(PROPOSAL_CAPABILITY);
    validate_sns_proposal_row(proposal).map_err(|reason| validator.invalid(reason))?;
    if proposal.proposal_id != requested_proposal_id {
        return Err(validator.invalid(format!(
            "returned proposal id {}, expected {requested_proposal_id}",
            proposal.proposal_id
        )));
    }
    Ok(())
}

fn validate_sns_proposal_source_rows(
    proposals: &[SnsProposalRow],
    requested_limit: u32,
    capability: &'static str,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new(capability);
    validator.row_limit(proposals.len(), requested_limit)?;
    validate_sns_proposal_rows(proposals).map_err(|reason| validator.invalid(reason))
}

/// Validate derived fields and proposal-id uniqueness within one row collection.
pub(in crate::sns::report) fn validate_sns_proposal_rows(
    proposals: &[SnsProposalRow],
) -> Result<(), String> {
    let mut proposal_ids = HashSet::new();
    for proposal in proposals {
        validate_sns_proposal_row(proposal)?;
        if !proposal_ids.insert(proposal.proposal_id) {
            return Err(format!("duplicate proposal id {}", proposal.proposal_id));
        }
    }
    Ok(())
}

fn validate_sns_proposal_row(proposal: &SnsProposalRow) -> Result<(), String> {
    let expected_action = SnsProposalAction::from_id(proposal.action_id);
    if proposal.action != expected_action {
        return Err(format!(
            "proposal {} action classification {} does not match raw action id {}",
            proposal.proposal_id, proposal.action, proposal.action_id
        ));
    }
    if proposal.ballot_count != proposal.ballots.len() {
        return Err(format!(
            "proposal {} ballot_count {} does not match {} ballot rows",
            proposal.proposal_id,
            proposal.ballot_count,
            proposal.ballots.len()
        ));
    }
    for ballot in &proposal.ballots {
        let expected_vote = SnsProposalVote::from_code(ballot.vote);
        if ballot.vote_text != expected_vote {
            return Err(format!(
                "proposal {} ballot {} vote classification {} does not match raw vote code {}",
                proposal.proposal_id, ballot.neuron_id, ballot.vote_text, ballot.vote
            ));
        }
    }
    Ok(())
}
