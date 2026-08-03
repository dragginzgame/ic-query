//! Module: nns::topology::report::capacity::summary
//!
//! Responsibility: summarize NNS topology capacity rows.
//! Does not own: row construction, sorting, source reads, or rendering.
//! Boundary: derives aggregate capacity totals and status.

use crate::nns::topology::report::{NnsTopologyAssessmentStatus, NnsTopologyCapacityRow};

///
/// CapacitySummary
///
/// Aggregate allowance, assignment, and availability totals for topology capacity.
///

pub(super) struct CapacitySummary {
    pub(super) status: NnsTopologyAssessmentStatus,
    pub(super) total_node_allowance: u64,
    pub(super) assigned_node_count: u64,
    pub(super) unknown_node_count_operator_count: usize,
    pub(super) available_node_slots: u64,
    pub(super) over_assigned_operator_count: usize,
    pub(super) over_assigned_node_count: u64,
}

pub(super) fn capacity_summary(capacity: &[NnsTopologyCapacityRow]) -> CapacitySummary {
    let total_node_allowance = capacity.iter().map(|row| row.node_allowance).sum();
    let assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.assigned_node_count)
        .sum();
    let unknown_node_count_operator_count = capacity
        .iter()
        .filter(|row| row.assigned_node_count.is_none())
        .count();
    let available_node_slots = capacity
        .iter()
        .filter_map(|row| row.available_node_slots)
        .sum();
    let over_assigned_operator_count = capacity
        .iter()
        .filter(|row| row.over_assigned_node_count.is_some_and(|count| count > 0))
        .count();
    let over_assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.over_assigned_node_count)
        .sum();
    let status = NnsTopologyAssessmentStatus::from_ok(
        over_assigned_operator_count == 0 && unknown_node_count_operator_count == 0,
    );

    CapacitySummary {
        status,
        total_node_allowance,
        assigned_node_count,
        unknown_node_count_operator_count,
        available_node_slots,
        over_assigned_operator_count,
        over_assigned_node_count,
    }
}
