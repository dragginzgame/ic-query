//! Module: nns::topology::report::capacity::sort
//!
//! Responsibility: sort NNS topology capacity rows for stable report output.
//! Does not own: capacity row construction, summary calculation, or rendering.
//! Boundary: defines the attention-first capacity row ordering.

use crate::nns::topology::report::NnsTopologyCapacityRow;

pub(super) fn sort_capacity_rows(capacity: &mut [NnsTopologyCapacityRow]) {
    capacity.sort_by(|left, right| {
        (
            capacity_status_rank(&left.status),
            left.available_node_slots.unwrap_or(u64::MAX),
            left.node_operator_principal.as_str(),
        )
            .cmp(&(
                capacity_status_rank(&right.status),
                right.available_node_slots.unwrap_or(u64::MAX),
                right.node_operator_principal.as_str(),
            ))
    });
}

fn capacity_status_rank(status: &str) -> u8 {
    match status {
        "over" => 0,
        "unknown" => 1,
        "full" => 2,
        "available" => 3,
        _ => 4,
    }
}
