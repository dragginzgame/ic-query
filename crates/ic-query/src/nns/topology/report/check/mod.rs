//! Module: nns::topology::report::check
//!
//! Responsibility: build derived NNS topology check reports.
//! Does not own: component collection, text rendering, or cache IO.
//! Boundary: projects component evidence directly into status fields and consistency checks.

use super::{
    NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus, NnsTopologyCheckReport,
    NnsTopologyCheckRow, TopologyInventoryReports, coverage::topology_join_coverage_counts,
    registry_versions::topology_registry_versions,
};

pub(super) fn topology_check_report_from_reports(
    network: String,
    source_endpoint: String,
    subnet_report: crate::subnet_catalog::SubnetCatalogListReport,
    reports: &TopologyInventoryReports,
) -> NnsTopologyCheckReport {
    let registry_versions = topology_registry_versions(&subnet_report, reports);
    let registry_source_count = registry_versions.len();
    let registry_version_min = registry_versions
        .iter()
        .map(|row| row.registry_version)
        .min();
    let registry_version_max = registry_versions
        .iter()
        .map(|row| row.registry_version)
        .max();
    let registry_versions_aligned = registry_version_min == registry_version_max;
    let stale_source_count = registry_versions
        .iter()
        .filter(|row| row.stale == Some(true))
        .count();
    let unknown_freshness_source_count = registry_versions
        .iter()
        .filter(|row| row.stale.is_none())
        .count();
    let (known_join_count, unknown_join_count) = join_totals(reports);
    let checks = vec![
        check_row("registry_versions", registry_versions_aligned),
        check_row(
            "cache_freshness",
            stale_source_count == 0 && unknown_freshness_source_count == 0,
        ),
        check_row("join_coverage", unknown_join_count == 0),
    ];
    let status = NnsTopologyAssessmentStatus::from_ok(
        checks
            .iter()
            .all(|row| row.status == NnsTopologyAssessmentStatus::Ok),
    );

    NnsTopologyCheckReport {
        schema_version: NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        registry_source_count,
        registry_version_min,
        registry_version_max,
        registry_versions_aligned,
        stale_source_count,
        unknown_freshness_source_count,
        subnet_catalog_stale: subnet_report.catalog_stale,
        subnet_catalog_stale_reason: subnet_report.stale_reason,
        known_join_count,
        unknown_join_count,
        checks,
    }
}

fn check_row(check: &str, is_ok: bool) -> NnsTopologyCheckRow {
    NnsTopologyCheckRow {
        check: check.to_string(),
        status: NnsTopologyAssessmentStatus::from_ok(is_ok),
    }
}

fn join_totals(reports: &TopologyInventoryReports) -> (usize, usize) {
    let counts = topology_join_coverage_counts(reports);
    [
        (
            counts.nodes_with_known_node_provider_count,
            counts.nodes_with_unknown_node_provider_count,
        ),
        (
            counts.nodes_with_known_node_operator_count,
            counts.nodes_with_unknown_node_operator_count,
        ),
        (
            counts.nodes_with_known_data_center_count,
            counts.nodes_with_unknown_data_center_count,
        ),
        (
            counts.node_operators_with_known_node_provider_count,
            counts.node_operators_with_unknown_node_provider_count,
        ),
        (
            counts.node_operators_with_known_data_center_count,
            counts.node_operators_with_unknown_data_center_count,
        ),
    ]
    .into_iter()
    .fold(
        (0_usize, 0_usize),
        |(known, unknown), (next_known, next_unknown)| {
            (
                known.saturating_add(next_known),
                unknown.saturating_add(next_unknown),
            )
        },
    )
}
