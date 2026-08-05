//! Module: nns::topology::report::check::checks
//!
//! Responsibility: construct NNS topology check rows.
//! Does not own: derived metric calculation, text rendering, or source reads.
//! Boundary: maps check metrics into user-facing check names and details.

use super::metrics::NnsTopologyCheckDerivedMetrics;
use crate::nns::topology::report::{
    NnsTopologyAssessmentStatus, NnsTopologyCheckRow, NnsTopologySummaryReport,
};

pub(super) fn topology_check_checks(
    summary: &NnsTopologySummaryReport,
    check: &NnsTopologyCheckDerivedMetrics,
) -> Vec<NnsTopologyCheckRow> {
    vec![
        check_row(
            "registry_versions",
            check.registry_versions_aligned,
            registry_version_detail(
                check.registry_source_count,
                check.registry_version_min,
                check.registry_version_max,
                check.registry_versions_aligned,
            ),
        ),
        check_row(
            "cache_freshness",
            check.stale_source_count == 0 && check.unknown_freshness_source_count == 0,
            cache_freshness_detail(
                check.stale_source_count,
                check.unknown_freshness_source_count,
                summary,
            ),
        ),
        check_row(
            "join_coverage",
            check.unknown_join_count == 0,
            format!(
                "{} known, {} unknown ({})",
                check.known_join_count, check.unknown_join_count, check.join_coverage
            ),
        ),
    ]
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
