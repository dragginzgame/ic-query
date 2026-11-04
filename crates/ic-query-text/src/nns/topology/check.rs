//! Module: nns::topology::check
//!
//! Responsibility: render NNS topology check reports as text.
//! Does not own: consistency-check derivation, cache loading, or JSON output.
//! Boundary: formats consistency-check rows and report status for humans.

use super::percent::coverage_percent_text;
use crate::text_value::table;
use ic_host_text::table::Alignment;

use ic_query::nns::topology::NnsTopologyCheckReport;
use ic_query::nns::topology::NnsTopologyCheckRow;

#[must_use]
/// Render the NNS topology check report as human-facing text.
pub fn nns_topology_check_report_text(report: &NnsTopologyCheckReport) -> String {
    render_check_table(report)
}

fn render_check_table(report: &NnsTopologyCheckReport) -> String {
    let headers = ["CHECK", "STATUS", "DETAIL"];
    let rows = report
        .checks
        .iter()
        .map(|row| {
            [
                row.check.clone(),
                row.status.as_str().to_string(),
                check_detail(report, row),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [Alignment::Left; 3];
    table(&headers, &rows, &alignments)
}

fn check_detail(report: &NnsTopologyCheckReport, row: &NnsTopologyCheckRow) -> String {
    match row.check.as_str() {
        "registry_versions" => match (
            report.registry_version_min,
            report.registry_version_max,
            report.registry_versions_aligned,
        ) {
            (Some(version), Some(_), true) => format!(
                "{} sources at registry version {version}",
                report.registry_source_count
            ),
            (Some(min), Some(max), false) => format!(
                "{} sources span registry versions {min}..{max}",
                report.registry_source_count
            ),
            _ => "no registry versions recorded".to_string(),
        },
        "cache_freshness" => cache_freshness_detail(report),
        "join_coverage" => format!(
            "{} known, {} unknown ({})",
            report.known_join_count,
            report.unknown_join_count,
            coverage_percent_text(report.known_join_count, report.unknown_join_count)
        ),
        _ => "-".to_string(),
    }
}

fn cache_freshness_detail(report: &NnsTopologyCheckReport) -> String {
    let stale = report.stale_source_count;
    let unknown = report.unknown_freshness_source_count;
    if stale == 0 && unknown == 0 {
        return "no stale topology sources".to_string();
    }
    if stale == 0 {
        return format!("{unknown} topology sources have no age policy");
    }
    if report.subnet_catalog_stale {
        return format!(
            "{stale} stale source, {unknown} without an age policy; subnet catalog {}",
            report.subnet_catalog_stale_reason
        );
    }
    format!("{stale} stale source, {unknown} without an age policy")
}
