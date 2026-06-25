//! Module: nns::topology::report::health
//!
//! Responsibility: build derived NNS topology health reports.
//! Does not own: topology summary construction, text rendering, or cache IO.
//! Boundary: turns summary metrics into status fields and health check rows.

mod checks;
mod metrics;

use super::{
    NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION, NnsTopologyHealthReport, NnsTopologySummaryReport,
};
use checks::topology_health_checks;
use metrics::topology_health_derived_metrics;

pub(super) fn topology_health_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyHealthReport {
    let health = topology_health_derived_metrics(&summary);
    let status = if health.registry_versions_aligned
        && health.stale_source_count == 0
        && health.unknown_join_count == 0
    {
        "ok"
    } else {
        "attention"
    }
    .to_string();
    let checks = topology_health_checks(&summary, &health);

    NnsTopologyHealthReport {
        schema_version: NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        status,
        registry_source_count: health.registry_source_count,
        registry_version_min: health.registry_version_min,
        registry_version_max: health.registry_version_max,
        registry_versions_aligned: health.registry_versions_aligned,
        stale_source_count: health.stale_source_count,
        subnet_catalog_stale: summary.subnet_catalog_stale,
        subnet_catalog_stale_reason: summary.subnet_catalog_stale_reason,
        known_join_count: health.known_join_count,
        unknown_join_count: health.unknown_join_count,
        join_coverage: health.join_coverage,
        checks,
    }
}
