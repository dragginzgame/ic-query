//! Module: sns::report::view::proposals
//!
//! Responsibility: apply SNS proposal list view ordering.
//! Does not own: proposal fetching, cache filtering, report assembly, or text rendering.
//! Boundary: sorts proposal rows without changing cache identity.

use crate::sns::report::{SnsProposalRow, SnsProposalsSort};

pub(in crate::sns::report) fn sort_sns_proposal_rows(
    proposals: &mut [SnsProposalRow],
    sort: SnsProposalsSort,
) {
    match sort {
        SnsProposalsSort::Api => {}
        SnsProposalsSort::Id => proposals.sort_by(|left, right| {
            right
                .proposal_id
                .cmp(&left.proposal_id)
                .then_with(|| right.created_at.cmp(&left.created_at))
        }),
        SnsProposalsSort::Created => proposals.sort_by(|left, right| {
            right
                .proposal_creation_timestamp_seconds
                .cmp(&left.proposal_creation_timestamp_seconds)
                .then_with(|| right.proposal_id.cmp(&left.proposal_id))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_id_sort_orders_newest_id_first() {
        let mut proposals = vec![
            proposal_row(2, 100),
            proposal_row(10, 50),
            proposal_row(1, 200),
        ];

        sort_sns_proposal_rows(&mut proposals, SnsProposalsSort::Id);

        assert_eq!(proposal_ids(&proposals), vec![10, 2, 1]);
    }

    fn proposal_ids(proposals: &[SnsProposalRow]) -> Vec<u64> {
        proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect()
    }

    fn proposal_row(proposal_id: u64, created_at_secs: u64) -> SnsProposalRow {
        SnsProposalRow {
            proposal_id: Some(proposal_id),
            action_id: 0,
            action: "motion".to_string(),
            title: String::new(),
            summary: String::new(),
            url: None,
            decision_state: "open".to_string(),
            reject_cost_e8s: 0,
            proposal_creation_timestamp_seconds: created_at_secs,
            created_at: created_at_secs.to_string(),
            decided_timestamp_seconds: None,
            decided_at: None,
            executed_timestamp_seconds: None,
            executed_at: None,
            failed_timestamp_seconds: None,
            failed_at: None,
            failure_reason: None,
            reward_event_round: 0,
            reward_event_end_timestamp_seconds: None,
            is_eligible_for_rewards: false,
            latest_tally: None,
            ballot_count: 0,
            ballots: Vec::new(),
            payload_text_rendering: None,
            proposer_neuron_id: None,
        }
    }
}
