use super::{NnsNodeListFilters, NnsNodeListReport};
use crate::subnet_catalog::canonical_principal_text;

pub(super) fn filter_node_list_report(
    mut report: NnsNodeListReport,
    filters: &NnsNodeListFilters,
) -> NnsNodeListReport {
    if filters.is_empty() {
        return report;
    }
    let subnet = filters.subnet.as_deref().map(PrincipalFilter::new);
    let node_provider = filters.node_provider.as_deref().map(PrincipalFilter::new);
    let node_operator = filters.node_operator.as_deref().map(PrincipalFilter::new);
    let data_center = filters
        .data_center
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase());
    report.nodes.retain(|node| {
        subnet
            .as_ref()
            .is_none_or(|filter| filter.matches(&node.subnet_principal))
            && filters
                .subnet_kind
                .is_none_or(|kind| node.subnet_kind == kind)
            && data_center.as_ref().is_none_or(|prefix| {
                !prefix.is_empty() && node.data_center_id.to_ascii_lowercase().starts_with(prefix)
            })
            && node_provider
                .as_ref()
                .is_none_or(|filter| filter.matches(&node.node_provider_principal))
            && node_operator
                .as_ref()
                .is_none_or(|filter| filter.matches(&node.node_operator_principal))
    });
    report.node_count = report.nodes.len();
    report
}

///
/// PrincipalFilter
///
/// A principal admitted once for exact matching, or a prefix that rejects empty input.
///

enum PrincipalFilter {
    Exact(String),
    Prefix(String),
}

impl PrincipalFilter {
    fn new(value: &str) -> Self {
        let value = value.trim();
        canonical_principal_text(value)
            .map_or_else(|_| Self::Prefix(value.to_ascii_lowercase()), Self::Exact)
    }

    fn matches(&self, value: &str) -> bool {
        match self {
            Self::Exact(principal) => value == principal,
            Self::Prefix(prefix) => !prefix.is_empty() && value.starts_with(prefix),
        }
    }
}
