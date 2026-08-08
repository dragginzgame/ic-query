//! Module: ic::node_status::counts
//!
//! Responsibility: canonical raw status and assignment-status aggregation.
//! Does not own: source validation, report projection, cache IO, or rendering.
//! Boundary: keeps every node-status view on one lossless counting flow.

use super::{
    IcNodeAssignmentStatusCounts, IcNodeOperationalStatus, IcNodeStatusCounts,
    IcNodeStatusGroupCounts, IcNodeStatusRow,
};

pub(in crate::ic) fn node_status_group_counts<'a>(
    nodes: impl Iterator<Item = &'a IcNodeStatusRow>,
) -> IcNodeStatusGroupCounts {
    let mut counts = IcNodeStatusGroupCounts::default();
    for node in nodes {
        let status = node.operational_status();
        increment_status(&mut counts.statuses, status);
        increment_status(
            assignment_bucket(&mut counts.assignment_statuses, node),
            status,
        );
    }
    counts
}

/// Count raw operational statuses without applying an assignment projection.
#[cfg(feature = "dashboard-host")]
pub fn node_status_counts<'a>(
    nodes: impl Iterator<Item = &'a IcNodeStatusRow>,
) -> IcNodeStatusCounts {
    let mut counts = IcNodeStatusCounts::default();
    for node in nodes {
        increment_status(&mut counts, node.operational_status());
    }
    counts
}

fn assignment_bucket<'a>(
    counts: &'a mut IcNodeAssignmentStatusCounts,
    node: &IcNodeStatusRow,
) -> &'a mut IcNodeStatusCounts {
    if node.subnet_id.is_some() {
        return &mut counts.assigned;
    }
    match node.node_type.as_str() {
        "UNASSIGNED" => &mut counts.unassigned,
        "API_BOUNDARY" => &mut counts.api_boundary,
        _ => &mut counts.unknown,
    }
}

const fn increment_status(counts: &mut IcNodeStatusCounts, status: IcNodeOperationalStatus) {
    counts.total += 1;
    match status {
        IcNodeOperationalStatus::Up => counts.up += 1,
        IcNodeOperationalStatus::Down => counts.down += 1,
        IcNodeOperationalStatus::Disabled => counts.disabled += 1,
        IcNodeOperationalStatus::Degraded => counts.degraded += 1,
        IcNodeOperationalStatus::Unknown => counts.unknown += 1,
    }
}
