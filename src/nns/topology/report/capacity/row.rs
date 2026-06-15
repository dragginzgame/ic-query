use super::super::{NnsTopologyCapacityRow, percent::ratio_percent_text};
use crate::nns::node_operator::report::NnsNodeOperatorRow;

pub(super) fn capacity_row_from_operator(operator: &NnsNodeOperatorRow) -> NnsTopologyCapacityRow {
    let assigned_node_count = operator.node_count.map(u64::from);
    let available_node_slots =
        assigned_node_count.map(|node_count| operator.node_allowance.saturating_sub(node_count));
    let over_assigned_node_count =
        assigned_node_count.map(|node_count| node_count.saturating_sub(operator.node_allowance));
    let utilization = assigned_node_count.map_or_else(
        || "-".to_string(),
        |node_count| {
            ratio_percent_text(u128::from(node_count), u128::from(operator.node_allowance))
        },
    );
    let status = if over_assigned_node_count.is_some_and(|count| count > 0) {
        "over"
    } else if available_node_slots == Some(0) {
        "full"
    } else if available_node_slots.is_some() {
        "available"
    } else {
        "unknown"
    }
    .to_string();

    NnsTopologyCapacityRow {
        node_operator_principal: operator.node_operator_principal.clone(),
        node_provider_principal: operator.node_provider_principal.clone(),
        data_center_id: operator.data_center_id.clone(),
        node_allowance: operator.node_allowance,
        assigned_node_count,
        available_node_slots,
        over_assigned_node_count,
        utilization,
        status,
    }
}
