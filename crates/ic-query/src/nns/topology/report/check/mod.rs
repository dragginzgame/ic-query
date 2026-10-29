//! Module: nns::topology::report::check
//!
//! Responsibility: build derived NNS topology check reports.
//! Does not own: topology summary construction, text rendering, or cache IO.
//! Boundary: turns summary metrics into status fields and consistency-check rows.

use super::{
    NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus, NnsTopologyCheckReport,
    NnsTopologyCheckRow, NnsTopologySummaryReport, percent::coverage_percent_text,
};

pub(super) fn topology_check_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyCheckReport {
    let registry_source_count = summary.registry_versions.len();
    let registry_version_min = summary
        .registry_versions
        .iter()
        .map(|row| row.registry_version)
        .min();
    let registry_version_max = summary
        .registry_versions
        .iter()
        .map(|row| row.registry_version)
        .max();
    let registry_versions_aligned = registry_version_min == registry_version_max;
    let stale_source_count = summary
        .registry_versions
        .iter()
        .filter(|row| row.stale == Some(true))
        .count();
    let unknown_freshness_source_count = summary
        .registry_versions
        .iter()
        .filter(|row| row.stale.is_none())
        .count();
    let (known_join_count, unknown_join_count) = summary.join_counts().into_iter().fold(
        (0_usize, 0_usize),
        |(known, unknown), (next_known, next_unknown)| {
            (
                known.saturating_add(next_known),
                unknown.saturating_add(next_unknown),
            )
        },
    );
    let join_coverage = coverage_percent_text(known_join_count, unknown_join_count);
    let checks = vec![
        check_row(
            "registry_versions",
            registry_versions_aligned,
            registry_version_detail(
                registry_source_count,
                registry_version_min,
                registry_version_max,
                registry_versions_aligned,
            ),
        ),
        check_row(
            "cache_freshness",
            stale_source_count == 0 && unknown_freshness_source_count == 0,
            cache_freshness_detail(stale_source_count, unknown_freshness_source_count, &summary),
        ),
        check_row(
            "join_coverage",
            unknown_join_count == 0,
            format!("{known_join_count} known, {unknown_join_count} unknown ({join_coverage})"),
        ),
    ];
    let status = NnsTopologyAssessmentStatus::from_ok(
        checks
            .iter()
            .all(|row| row.status == NnsTopologyAssessmentStatus::Ok),
    );

    NnsTopologyCheckReport {
        schema_version: NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        status,
        registry_source_count,
        registry_version_min,
        registry_version_max,
        registry_versions_aligned,
        stale_source_count,
        unknown_freshness_source_count,
        subnet_catalog_stale: summary.subnet_catalog_stale,
        subnet_catalog_stale_reason: summary.subnet_catalog_stale_reason,
        known_join_count,
        unknown_join_count,
        join_coverage,
        checks,
    }
}

fn check_row(check: &str, is_ok: bool, detail: String) -> NnsTopologyCheckRow {
    NnsTopologyCheckRow {
        check: check.to_string(),
        status: NnsTopologyAssessmentStatus::from_ok(is_ok),
        detail,
    }
}

fn registry_version_detail(
    source_count: usize,
    min: Option<u64>,
    max: Option<u64>,
    aligned: bool,
) -> String {
    match (min, max, aligned) {
        (Some(version), Some(_), true) => {
            format!("{source_count} sources at registry version {version}")
        }
        (Some(min), Some(max), false) => {
            format!("{source_count} sources span registry versions {min}..{max}")
        }
        _ => "no registry versions recorded".to_string(),
    }
}

fn cache_freshness_detail(
    stale_source_count: usize,
    unknown_source_count: usize,
    summary: &NnsTopologySummaryReport,
) -> String {
    if stale_source_count == 0 && unknown_source_count == 0 {
        return "no stale topology sources".to_string();
    }
    if stale_source_count == 0 {
        return format!("{unknown_source_count} topology sources have no age policy");
    }
    if summary.subnet_catalog_stale {
        return format!(
            "{stale_source_count} stale source, {unknown_source_count} without an age policy; subnet catalog {}",
            summary.subnet_catalog_stale_reason,
        );
    }
    format!("{stale_source_count} stale source, {unknown_source_count} without an age policy")
}
