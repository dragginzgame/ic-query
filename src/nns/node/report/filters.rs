use super::{NnsNodeListFilters, NnsNodeListReport, NnsNodeRow};
use crate::subnet_catalog::canonical_principal_text;

pub(super) fn filter_node_list_report(
    mut report: NnsNodeListReport,
    filters: &NnsNodeListFilters,
) -> NnsNodeListReport {
    if filters.is_empty() {
        return report;
    }
    report
        .nodes
        .retain(|node| node_matches_filters(node, filters));
    report.node_count = report.nodes.len();
    report
}

fn node_matches_filters(node: &NnsNodeRow, filters: &NnsNodeListFilters) -> bool {
    filters
        .subnet
        .as_deref()
        .is_none_or(|filter| principal_filter_matches(&node.subnet_principal, filter))
        && filters
            .subnet_kind
            .as_deref()
            .is_none_or(|filter| text_filter_equals(&node.subnet_kind, filter))
        && filters
            .data_center
            .as_deref()
            .is_none_or(|filter| text_filter_starts_with(&node.data_center_id, filter))
        && filters
            .node_provider
            .as_deref()
            .is_none_or(|filter| principal_filter_matches(&node.node_provider_principal, filter))
        && filters
            .node_operator
            .as_deref()
            .is_none_or(|filter| principal_filter_matches(&node.node_operator_principal, filter))
}

fn principal_filter_matches(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    if let Ok(principal) = canonical_principal_text(filter) {
        value == principal
    } else {
        value.starts_with(&filter.to_ascii_lowercase())
    }
}

fn text_filter_starts_with(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    value
        .to_ascii_lowercase()
        .starts_with(&filter.to_ascii_lowercase())
}

fn text_filter_equals(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    value.eq_ignore_ascii_case(filter)
}

fn non_empty_filter(filter: &str) -> Option<&str> {
    let filter = filter.trim();
    (!filter.is_empty()).then_some(filter)
}
