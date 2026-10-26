//! Module: nns::topology::report::text::common
//!
//! Responsibility: provide shared NNS topology text table helpers.
//! Does not own: report construction, source reads, or command parsing.
//! Boundary: centralizes repeated table formatting for topology renderers.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::table::{Alignment, render};

use crate::{
    nns::topology::report::{NnsTopologyRegistryVersionRow, percent::coverage_percent_text},
    text_value::yes_no,
};

pub(super) fn render_join_coverage_table(counts: [(usize, usize); 5]) -> String {
    let headers = ["RELATION", "KNOWN", "UNKNOWN", "COVERAGE"];
    let labels = [
        "nodes -> node providers",
        "nodes -> node operators",
        "nodes -> data centers",
        "node operators -> node providers",
        "node operators -> data centers",
    ];
    let rows = counts
        .iter()
        .zip(labels)
        .map(|((known, unknown), link)| {
            [
                link.to_string(),
                known.to_string(),
                unknown.to_string(),
                coverage_percent_text(*known, *unknown),
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
