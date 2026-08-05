//! Module: nns::topology::report::check
//!
//! Responsibility: build derived NNS topology check reports.
//! Does not own: topology summary construction, text rendering, or cache IO.
//! Boundary: turns summary metrics into status fields and consistency-check rows.

mod checks;
mod metrics;

use super::{
    NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus, NnsTopologyCheckReport,
    NnsTopologySummaryReport,
};
use checks::topology_check_checks;
use metrics::topology_check_derived_metrics;

pub(super) fn topology_check_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyCheckReport {
    let check = topology_check_derived_metrics(&summary);
    let status = NnsTopologyAssessmentStatus::from_ok(
        check.registry_versions_aligned
            && check.stale_source_count == 0
            && check.unknown_freshness_source_count == 0
            && check.unknown_join_count == 0,
    );
    let checks = topology_check_checks(&summary, &check);

    NnsTopologyCheckReport {
        schema_version: NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        status,
        registry_source_count: check.registry_source_count,
        registry_version_min: check.registry_version_min,
        registry_version_max: check.registry_version_max,
        registry_versions_aligned: check.registry_versions_aligned,
        stale_source_count: check.stale_source_count,
        unknown_freshness_source_count: check.unknown_freshness_source_count,
        subnet_catalog_stale: summary.subnet_catalog_stale,
        subnet_catalog_stale_reason: summary.subnet_catalog_stale_reason,
        known_join_count: check.known_join_count,
        unknown_join_count: check.unknown_join_count,
        join_coverage: check.join_coverage,
        checks,
    }
}
