//! Module: sns::report::source::model::proposals
//!
//! Responsibility: source-layer SNS proposal models.
//! Does not own: governance transport, proposal conversion, or rendering.
//! Boundary: carries converted proposal rows from sources to builders.

use super::validation::SnsSourceValidator;
use crate::sns::report::{SnsHostError, SnsProposalAction, SnsProposalRow, SnsProposalVote};
use std::collections::HashSet;

const PROPOSAL_CAPABILITY: &str = "SNS proposal";
const PROPOSALS_CAPABILITY: &str = "SNS proposals";
const PROPOSAL_PAGE_CAPABILITY: &str = "SNS proposal page";

///
/// MainnetSnsProposals
///
/// Source-layer bounded SNS proposal listing.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposals {
    pub proposals: Vec<SnsProposalRow>,
}

///
/// MainnetSnsProposalPage
///
/// Source-layer SNS proposal page used by complete snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposalPage {
    pub proposals: Vec<SnsProposalRow>,
}

///
/// MainnetSnsProposal
///
/// Source-layer SNS proposal detail result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposal {
    pub proposal: SnsProposalRow,
}

/// Validate one bounded proposal result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposals(
    proposals: &MainnetSnsProposals,
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    validate_sns_proposal_source_rows(&proposals.proposals, requested_limit, PROPOSALS_CAPABILITY)
}

/// Validate one proposal page returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposal_page(
    page: &MainnetSnsProposalPage,
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    validate_sns_proposal_source_rows(&page.proposals, requested_limit, PROPOSAL_PAGE_CAPABILITY)
}

/// Validate one exact proposal returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_proposal(
    proposal: &MainnetSnsProposal,
    requested_proposal_id: u64,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new(PROPOSAL_CAPABILITY);
    validate_sns_proposal_row(&proposal.proposal).map_err(|reason| validator.invalid(reason))?;
    if proposal.proposal.proposal_id != requested_proposal_id {
        return Err(validator.invalid(format!(
            "returned proposal id {}, expected {requested_proposal_id}",
            proposal.proposal.proposal_id
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
    if proposals.len() > requested_limit as usize {
        return Err(validator.invalid(format!(
            "returned {} rows for requested limit {requested_limit}",
            proposals.len()
        )));
    }
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
