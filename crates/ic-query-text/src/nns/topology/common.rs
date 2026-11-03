//! Module: nns::topology::common
//!
//! Responsibility: provide shared NNS topology text table helpers.
//! Does not own: report construction, source reads, or command parsing.
//! Boundary: centralizes repeated table formatting for topology renderers.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use super::percent::coverage_percent_text;
use crate::text_value::yes_no;
use ic_query::nns::topology::NnsTopologyJoinCoverageCounts;
use ic_query::nns::topology::NnsTopologyRegistryVersionRow;

pub(super) fn render_join_coverage_table(counts: &NnsTopologyJoinCoverageCounts) -> String {
    let headers = ["RELATION", "KNOWN", "UNKNOWN", "COVERAGE"];
    let rows = relationships(counts)
        .into_iter()
        .map(|(link, known, unknown)| {
            [
                link.to_string(),
                known.to_string(),
                unknown.to_string(),
                coverage_percent_text(known, unknown),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
    ];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}

pub(super) fn render_registry_version_table(rows: &[NnsTopologyRegistryVersionRow]) -> String {
    let headers = ["SOURCE", "VERSION", "FETCHED_AT", "STALE", "ENDPOINT"];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.source.clone(),
                row.registry_version.to_string(),
                row.fetched_at.clone(),
                row.stale
                    .map_or_else(|| "-".to_string(), |stale| yes_no(stale).to_string()),
                row.source_endpoint.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
    ];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}

const fn relationships(
    counts: &NnsTopologyJoinCoverageCounts,
) -> [(&'static str, usize, usize); 5] {
    [
        (
            "nodes -> node providers",
            counts.nodes_with_known_node_provider_count,
            counts.nodes_with_unknown_node_provider_count,
        ),
        (
            "nodes -> node operators",
            counts.nodes_with_known_node_operator_count,
            counts.nodes_with_unknown_node_operator_count,
        ),
        (
            "nodes -> data centers",
            counts.nodes_with_known_data_center_count,
            counts.nodes_with_unknown_data_center_count,
        ),
        (
            "node operators -> node providers",
            counts.node_operators_with_known_node_provider_count,
            counts.node_operators_with_unknown_node_provider_count,
        ),
        (
            "node operators -> data centers",
            counts.node_operators_with_known_data_center_count,
            counts.node_operators_with_unknown_data_center_count,
        ),
    ]
}
