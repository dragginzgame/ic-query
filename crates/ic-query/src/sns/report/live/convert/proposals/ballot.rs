//! Module: sns::report::live::convert::proposals::ballot
//!
//! Responsibility: convert SNS governance proposal ballots.
//! Does not own: governance transport, proposal row assembly, or rendering.
//! Boundary: maps live ballot tuples into report ballot rows.

use super::timestamp::optional_timestamp_text;
use crate::sns::report::{SnsProposalBallotRow, SnsProposalVote, live::types::SnsGovernanceBallot};

/// Convert one SNS governance ballot tuple into a report ballot row.
pub(super) fn sns_proposal_ballot_row(
    (neuron_id, ballot): (String, SnsGovernanceBallot),
) -> SnsProposalBallotRow {
    SnsProposalBallotRow {
        neuron_id,
        vote: ballot.vote,
        vote_text: SnsProposalVote::from_code(ballot.vote),
        cast_timestamp_seconds: ballot.cast_timestamp_seconds,
        cast_at: optional_timestamp_text(ballot.cast_timestamp_seconds),
        voting_power: ballot.voting_power,
    }
}
