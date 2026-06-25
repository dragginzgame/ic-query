//! Module: nns::topology::report::health::metrics
//!
//! Responsibility: derive aggregate NNS topology health metrics.
//! Does not own: check row wording, text rendering, or report serialization.
//! Boundary: calculates registry, staleness, and join-coverage health inputs.

use crate::nns::topology::report::{NnsTopologySummaryReport, percent::coverage_percent_text};

pub(super) struct NnsTopologyHealthDerivedMetrics {
    pub(super) registry_source_count: usize,
    pub(super) registry_version_min: Option<u64>,
    pub(super) registry_version_max: Option<u64>,
    pub(super) registry_versions_aligned: bool,
    pub(super) stale_source_count: usize,
    pub(super) known_join_count: usize,
    pub(super) unknown_join_count: usize,
    pub(super) join_coverage: String,
}

pub(super) fn topology_health_derived_metrics(
    summary: &NnsTopologySummaryReport,
) -> NnsTopologyHealthDerivedMetrics {
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
    let known_join_count = known_join_count(summary);
    let unknown_join_count = unknown_join_count(summary);

    NnsTopologyHealthDerivedMetrics {
        registry_source_count: summary.registry_versions.len(),
        registry_version_min,
        registry_version_max,
        registry_versions_aligned: registry_version_min == registry_version_max,
        stale_source_count: summary
            .registry_versions
            .iter()
            .filter(|row| row.stale == Some(true))
            .count(),
        known_join_count,
        unknown_join_count,
        join_coverage: coverage_percent_text(known_join_count, unknown_join_count),
    }
}

const fn known_join_count(report: &NnsTopologySummaryReport) -> usize {
    report
        .nodes_with_known_node_provider_count
        .saturating_add(report.nodes_with_known_node_operator_count)
        .saturating_add(report.nodes_with_known_data_center_count)
        .saturating_add(report.node_operators_with_known_node_provider_count)
        .saturating_add(report.node_operators_with_known_data_center_count)
}

const fn unknown_join_count(report: &NnsTopologySummaryReport) -> usize {
    report
        .nodes_with_unknown_node_provider_count
        .saturating_add(report.nodes_with_unknown_node_operator_count)
        .saturating_add(report.nodes_with_unknown_data_center_count)
        .saturating_add(report.node_operators_with_unknown_node_provider_count)
        .saturating_add(report.node_operators_with_unknown_data_center_count)
}
