//! Module: nns::topology::report::health::checks
//!
//! Responsibility: construct NNS topology health check rows.
//! Does not own: derived metric calculation, text rendering, or source reads.
//! Boundary: maps health metrics into user-facing check names and details.

use super::metrics::NnsTopologyHealthDerivedMetrics;
use crate::nns::topology::report::{NnsTopologyHealthCheckRow, NnsTopologySummaryReport};

pub(super) fn topology_health_checks(
    summary: &NnsTopologySummaryReport,
    health: &NnsTopologyHealthDerivedMetrics,
) -> Vec<NnsTopologyHealthCheckRow> {
    vec![
        health_check_row(
            "registry_versions",
            health.registry_versions_aligned,
            registry_version_detail(
                health.registry_source_count,
                health.registry_version_min,
                health.registry_version_max,
                health.registry_versions_aligned,
            ),
        ),
        health_check_row(
            "cache_freshness",
            health.stale_source_count == 0,
            cache_freshness_detail(health.stale_source_count, summary),
        ),
        health_check_row(
            "join_coverage",
            health.unknown_join_count == 0,
            format!(
                "{} known, {} unknown ({})",
                health.known_join_count, health.unknown_join_count, health.join_coverage
            ),
        ),
    ]
}

fn health_check_row(check: &str, is_ok: bool, detail: String) -> NnsTopologyHealthCheckRow {
    NnsTopologyHealthCheckRow {
        check: check.to_string(),
        status: if is_ok { "ok" } else { "attention" }.to_string(),
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

fn cache_freshness_detail(stale_source_count: usize, summary: &NnsTopologySummaryReport) -> String {
    if stale_source_count == 0 {
        return "no stale topology sources".to_string();
    }
    if summary.subnet_catalog_stale {
        return format!(
            "{stale_source_count} stale source; subnet catalog {}",
            summary.subnet_catalog_stale_reason
        );
    }
    format!("{stale_source_count} stale source")
}
