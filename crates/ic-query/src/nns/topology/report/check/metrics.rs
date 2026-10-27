//! Module: nns::topology::report::check::metrics
//!
//! Responsibility: derive aggregate NNS topology check metrics.
//! Does not own: check row wording, text rendering, or report serialization.
//! Boundary: calculates registry, staleness, and join-coverage check inputs.

use crate::nns::topology::report::{NnsTopologySummaryReport, percent::coverage_percent_text};

///
/// NnsTopologyCheckDerivedMetrics
///
/// Derived registry, staleness, and join metrics used by topology consistency checks.
///

pub(super) struct NnsTopologyCheckDerivedMetrics {
    pub(super) registry_source_count: usize,
    pub(super) registry_version_min: Option<u64>,
    pub(super) registry_version_max: Option<u64>,
    pub(super) registry_versions_aligned: bool,
    pub(super) stale_source_count: usize,
    pub(super) unknown_freshness_source_count: usize,
    pub(super) known_join_count: usize,
    pub(super) unknown_join_count: usize,
    pub(super) join_coverage: String,
}

pub(super) fn topology_check_derived_metrics(
    summary: &NnsTopologySummaryReport,
) -> NnsTopologyCheckDerivedMetrics {
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
    let (known_join_count, unknown_join_count) = summary.join_counts().into_iter().fold(
        (0_usize, 0_usize),
        |(known, unknown), (next_known, next_unknown)| {
            (
                known.saturating_add(next_known),
                unknown.saturating_add(next_unknown),
            )
        },
    );

    NnsTopologyCheckDerivedMetrics {
        registry_source_count: summary.registry_versions.len(),
        registry_version_min,
        registry_version_max,
        registry_versions_aligned: registry_version_min == registry_version_max,
        stale_source_count: summary
            .registry_versions
            .iter()
            .filter(|row| row.stale == Some(true))
            .count(),
        unknown_freshness_source_count: summary
            .registry_versions
            .iter()
            .filter(|row| row.stale.is_none())
            .count(),
        known_join_count,
        unknown_join_count,
        join_coverage: coverage_percent_text(known_join_count, unknown_join_count),
    }
}
