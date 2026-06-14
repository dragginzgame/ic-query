use super::super::{NnsTopologyRegistryVersionRow, health::coverage_percent_text};
use crate::{
    nns::render::yes_no,
    table::{ColumnAlign, render_table},
};

pub(super) fn render_join_coverage_table(rows: &[(&str, usize, usize)]) -> String {
    let headers = ["RELATION", "KNOWN", "UNKNOWN", "COVERAGE"];
    let rows = rows
        .iter()
        .map(|(link, known, unknown)| {
            [
                (*link).to_string(),
                known.to_string(),
                unknown.to_string(),
                coverage_percent_text(*known, *unknown),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}

pub(super) fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
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
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}
