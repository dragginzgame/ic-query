use super::{labels::ballot_vote_text, timestamp::optional_timestamp_text};
use crate::sns::report::{SnsProposalBallotRow, live::types::SnsGovernanceBallot};

pub(super) fn sns_proposal_ballot_row(
    (neuron_id, ballot): (String, SnsGovernanceBallot),
) -> SnsProposalBallotRow {
    SnsProposalBallotRow {
        neuron_id,
        vote: ballot.vote,
        vote_text: ballot_vote_text(ballot.vote),
        cast_timestamp_seconds: ballot.cast_timestamp_seconds,
        cast_at: optional_timestamp_text(ballot.cast_timestamp_seconds),
        voting_power: ballot.voting_power,
    }
}
