use super::{
    NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION, NnsTopologyHealthCheckRow, NnsTopologyHealthReport,
    NnsTopologySummaryReport,
};

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

struct NnsTopologyHealthDerivedMetrics {
    registry_source_count: usize,
    registry_version_min: Option<u64>,
    registry_version_max: Option<u64>,
    registry_versions_aligned: bool,
    stale_source_count: usize,
    known_join_count: usize,
    unknown_join_count: usize,
    join_coverage: String,
}

fn topology_health_derived_metrics(
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

fn topology_health_checks(
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

pub(super) fn coverage_percent_text(known: usize, unknown: usize) -> String {
    let total = known.saturating_add(unknown);
    if total == 0 {
        return "-".to_string();
    }
    let tenths = known.saturating_mul(1000).saturating_add(total / 2) / total;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}
