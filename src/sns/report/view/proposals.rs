//! Module: sns::report::view::proposals
//!
//! Responsibility: apply SNS proposal list view ordering.
//! Does not own: proposal fetching, cache filtering, report assembly, or text rendering.
//! Boundary: sorts proposal rows without changing cache identity.

use crate::sns::report::{
    SnsProposalRow, SnsProposalSortDirection, SnsProposalStatusFilter, SnsProposalsSort,
};
use std::cmp::Ordering;

pub(in crate::sns::report) fn proposal_matches_before(
    proposal: &SnsProposalRow,
    before_proposal_id: Option<u64>,
) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

pub(in crate::sns::report) fn proposal_matches_status(
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

pub(in crate::sns::report) fn sort_sns_proposal_rows(
    proposals: &mut [SnsProposalRow],
    sort: SnsProposalsSort,
    direction: SnsProposalSortDirection,
) {
    match sort {
        SnsProposalsSort::Api => {}
        SnsProposalsSort::Id => {
            sort_by_optional_u64(proposals, direction, |proposal| proposal.proposal_id);
        }
        SnsProposalsSort::Created => sort_by_optional_u64(proposals, direction, |proposal| {
            Some(proposal.proposal_creation_timestamp_seconds)
        }),
        SnsProposalsSort::Decided => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.decided_timestamp_seconds
        }),
        SnsProposalsSort::Executed => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.executed_timestamp_seconds
        }),
        SnsProposalsSort::Failed => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.failed_timestamp_seconds
        }),
    }
}

fn sort_by_optional_u64(
    proposals: &mut [SnsProposalRow],
    direction: SnsProposalSortDirection,
    key: impl Fn(&SnsProposalRow) -> Option<u64>,
) {
    proposals.sort_by(|left, right| {
        compare_optional_u64(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn compare_optional_u64(
    left: Option<u64>,
    right: Option<u64>,
    direction: SnsProposalSortDirection,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_u64(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_u64(left: u64, right: u64, direction: SnsProposalSortDirection) -> Ordering {
    match direction {
        SnsProposalSortDirection::Asc => left.cmp(&right),
        SnsProposalSortDirection::Desc => right.cmp(&left),
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

        sort_sns_proposal_rows(
            &mut proposals,
            SnsProposalsSort::Id,
            SnsProposalSortDirection::Desc,
        );

        assert_eq!(proposal_ids(&proposals), vec![10, 2, 1]);
    }

    #[test]
    fn proposal_decided_sort_orders_newest_decision_first_and_open_last() {
        let mut proposals = vec![
            proposal_row_with_decision(2, Some(100)),
            proposal_row_with_decision(10, None),
            proposal_row_with_decision(1, Some(200)),
        ];

        sort_sns_proposal_rows(
            &mut proposals,
            SnsProposalsSort::Decided,
            SnsProposalSortDirection::Desc,
        );

        assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
    }

    #[test]
    fn proposal_decided_ascending_sort_orders_oldest_decision_first_and_open_last() {
        let mut proposals = vec![
            proposal_row_with_decision(2, Some(100)),
            proposal_row_with_decision(10, None),
            proposal_row_with_decision(1, Some(200)),
        ];

        sort_sns_proposal_rows(
            &mut proposals,
            SnsProposalsSort::Decided,
            SnsProposalSortDirection::Asc,
        );

        assert_eq!(proposal_ids(&proposals), vec![2, 1, 10]);
    }

    #[test]
    fn proposal_executed_sort_orders_newest_execution_first_and_unexecuted_last() {
        let mut proposals = vec![
            proposal_row_with_execution(2, Some(100)),
            proposal_row_with_execution(10, None),
            proposal_row_with_execution(1, Some(200)),
        ];

        sort_sns_proposal_rows(
            &mut proposals,
            SnsProposalsSort::Executed,
            SnsProposalSortDirection::Desc,
        );

        assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
    }

    #[test]
    fn proposal_failed_sort_orders_newest_failure_first_and_non_failed_last() {
        let mut proposals = vec![
            proposal_row_with_failure(2, Some(100)),
            proposal_row_with_failure(10, None),
            proposal_row_with_failure(1, Some(200)),
        ];

        sort_sns_proposal_rows(
            &mut proposals,
            SnsProposalsSort::Failed,
            SnsProposalSortDirection::Desc,
        );

        assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
    }

    #[test]
    fn proposal_before_filter_requires_lower_present_id() {
        assert!(proposal_matches_before(&proposal_row(9, 100), Some(10)));
        assert!(!proposal_matches_before(&proposal_row(10, 100), Some(10)));
        assert!(proposal_matches_before(&proposal_without_id(), None));
        assert!(!proposal_matches_before(&proposal_without_id(), Some(10)));
    }

    #[test]
    fn proposal_status_filter_matches_cache_backed_statuses() {
        assert!(proposal_matches_status(
            &proposal_with_decision_state("executed"),
            SnsProposalStatusFilter::Executed
        ));
        assert!(!proposal_matches_status(
            &proposal_with_decision_state("open"),
            SnsProposalStatusFilter::Failed
        ));
        assert!(!proposal_matches_status(
            &proposal_with_decision_state("adopted"),
            SnsProposalStatusFilter::Adopted
        ));
    }

    fn proposal_ids(proposals: &[SnsProposalRow]) -> Vec<u64> {
        proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect()
    }

    fn proposal_without_id() -> SnsProposalRow {
        SnsProposalRow {
            proposal_id: None,
            ..proposal_row(1, 100)
        }
    }

    fn proposal_with_decision_state(decision_state: &str) -> SnsProposalRow {
        SnsProposalRow {
            decision_state: decision_state.to_string(),
            ..proposal_row(1, 100)
        }
    }

    fn proposal_row_with_decision(
        proposal_id: u64,
        decided_timestamp_seconds: Option<u64>,
    ) -> SnsProposalRow {
        SnsProposalRow {
            proposal_id: Some(proposal_id),
            decided_timestamp_seconds,
            decided_at: decided_timestamp_seconds.map(|value| value.to_string()),
            ..proposal_row(proposal_id, 100)
        }
    }

    fn proposal_row_with_execution(
        proposal_id: u64,
        executed_timestamp_seconds: Option<u64>,
    ) -> SnsProposalRow {
        SnsProposalRow {
            proposal_id: Some(proposal_id),
            executed_timestamp_seconds,
            executed_at: executed_timestamp_seconds.map(|value| value.to_string()),
            ..proposal_row(proposal_id, 100)
        }
    }

    fn proposal_row_with_failure(
        proposal_id: u64,
        failed_timestamp_seconds: Option<u64>,
    ) -> SnsProposalRow {
        SnsProposalRow {
            proposal_id: Some(proposal_id),
            failed_timestamp_seconds,
            failed_at: failed_timestamp_seconds.map(|value| value.to_string()),
            ..proposal_row(proposal_id, 100)
        }
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
