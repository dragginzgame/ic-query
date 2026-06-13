use super::*;
use crate::{
    nns::render::{compact_text, yes_no},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_summary_report_text(report: &NnsTopologySummaryReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "topology: {} subnets {} nodes {} node_operators {} node_providers {} data_centers {}",
        report.network,
        report.subnet_count,
        report.node_count,
        report.node_operator_count,
        report.node_provider_count,
        report.data_center_count
    ));
    lines.push(String::new());
    lines.push(render_count_table(report));
    lines.push(String::new());
    lines.push(render_kind_table(report));
    lines.push(String::new());
    lines.push(render_summary_join_coverage_table(report));
    lines.push(String::new());
    lines.push(render_registry_version_table(&report.registry_versions));
    lines.join("\n")
}

#[must_use]
pub fn nns_topology_coverage_report_text(report: &NnsTopologyCoverageReport) -> String {
    let lines = [
        render_coverage_count_table(report),
        String::new(),
        render_coverage_join_coverage_table(report),
    ];
    lines.join("\n")
}

#[must_use]
pub fn nns_topology_versions_report_text(report: &NnsTopologyVersionsReport) -> String {
    render_registry_version_table(&report.registry_versions)
}

#[must_use]
pub fn nns_topology_health_report_text(report: &NnsTopologyHealthReport) -> String {
    render_health_check_table(&report.checks)
}

#[must_use]
pub fn nns_topology_gaps_report_text(report: &NnsTopologyGapsReport) -> String {
    if report.gaps.is_empty() {
        return render_gaps_status_table(report);
    }
    render_gaps_table(&report.gaps)
}

#[must_use]
pub fn nns_topology_capacity_report_text(report: &NnsTopologyCapacityReport) -> String {
    let lines = [
        render_capacity_summary_table(report),
        String::new(),
        render_capacity_attention_table(report),
    ];
    lines.join("\n")
}

#[must_use]
pub fn nns_topology_regions_report_text(report: &NnsTopologyRegionsReport) -> String {
    render_regions_table(&report.regions)
}

#[must_use]
pub fn nns_topology_providers_report_text(report: &NnsTopologyProvidersReport) -> String {
    render_providers_table(&report.providers)
}

#[must_use]
pub fn nns_topology_refresh_report_text(report: &NnsTopologyRefreshReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "topology_refresh: {} components {} wrote {} replaced {} dry_run {}",
        report.network,
        report.component_count,
        report.wrote_cache_count,
        report.replaced_existing_cache_count,
        yes_no(report.dry_run)
    ));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(render_refresh_table(report));
    lines.join("\n")
}

fn render_count_table(report: &NnsTopologySummaryReport) -> String {
    let headers = ["METRIC", "COUNT"];
    let rows = [
        ["subnets".to_string(), report.subnet_count.to_string()],
        [
            "routing_ranges".to_string(),
            report.routing_range_count.to_string(),
        ],
        ["nodes".to_string(), report.node_count.to_string()],
        [
            "node_operators".to_string(),
            report.node_operator_count.to_string(),
        ],
        [
            "node_providers".to_string(),
            report.node_provider_count.to_string(),
        ],
        [
            "data_centers".to_string(),
            report.data_center_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_coverage_count_table(report: &NnsTopologyCoverageReport) -> String {
    let headers = ["FIELD", "VALUE"];
    let rows = [
        ["network".to_string(), report.network.clone()],
        ["nodes".to_string(), report.node_count.to_string()],
        [
            "node_operators".to_string(),
            report.node_operator_count.to_string(),
        ],
        [
            "node_providers".to_string(),
            report.node_provider_count.to_string(),
        ],
        [
            "data_centers".to_string(),
            report.data_center_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_kind_table(report: &NnsTopologySummaryReport) -> String {
    let headers = ["KIND", "SUBNETS", "NODES"];
    let rows = [
        [
            NNS_NODE_SUBNET_KIND_APPLICATION.to_string(),
            report.application_subnet_count.to_string(),
            report.application_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_CLOUD_ENGINE.to_string(),
            report.cloud_engine_subnet_count.to_string(),
            report.cloud_engine_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_SYSTEM.to_string(),
            report.system_subnet_count.to_string(),
            report.system_node_count.to_string(),
        ],
        [
            NNS_NODE_SUBNET_KIND_UNKNOWN.to_string(),
            report.unknown_subnet_count.to_string(),
            report.unknown_node_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_summary_join_coverage_table(report: &NnsTopologySummaryReport) -> String {
    render_join_coverage_table(&[
        (
            "nodes -> node providers",
            report.nodes_with_known_node_provider_count,
            report.nodes_with_unknown_node_provider_count,
        ),
        (
            "nodes -> node operators",
            report.nodes_with_known_node_operator_count,
            report.nodes_with_unknown_node_operator_count,
        ),
        (
            "nodes -> data centers",
            report.nodes_with_known_data_center_count,
            report.nodes_with_unknown_data_center_count,
        ),
        (
            "node operators -> node providers",
            report.node_operators_with_known_node_provider_count,
            report.node_operators_with_unknown_node_provider_count,
        ),
        (
            "node operators -> data centers",
            report.node_operators_with_known_data_center_count,
            report.node_operators_with_unknown_data_center_count,
        ),
    ])
}

fn render_coverage_join_coverage_table(report: &NnsTopologyCoverageReport) -> String {
    render_join_coverage_table(&[
        (
            "nodes -> node providers",
            report.nodes_with_known_node_provider_count,
            report.nodes_with_unknown_node_provider_count,
        ),
        (
            "nodes -> node operators",
            report.nodes_with_known_node_operator_count,
            report.nodes_with_unknown_node_operator_count,
        ),
        (
            "nodes -> data centers",
            report.nodes_with_known_data_center_count,
            report.nodes_with_unknown_data_center_count,
        ),
        (
            "node operators -> node providers",
            report.node_operators_with_known_node_provider_count,
            report.node_operators_with_unknown_node_provider_count,
        ),
        (
            "node operators -> data centers",
            report.node_operators_with_known_data_center_count,
            report.node_operators_with_unknown_data_center_count,
        ),
    ])
}

fn render_join_coverage_table(rows: &[(&str, usize, usize)]) -> String {
    let headers = ["RELATION", "KNOWN", "UNKNOWN", "COVERAGE"];
    let rows = rows
        .iter()
        .map(|(link, known, unknown)| {
            [
                (*link).to_string(),
                known.to_string(),
                unknown.to_string(),
                coverage_percent_text(*known, *unknown),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}

fn render_health_check_table(rows: &[NnsTopologyHealthCheckRow]) -> String {
    let headers = ["CHECK", "STATUS", "DETAIL"];
    let rows = rows
        .iter()
        .map(|row| [row.check.clone(), row.status.clone(), row.detail.clone()])
        .collect::<Vec<_>>();
    let alignments = [ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left];
    render_table(&headers, &rows, &alignments)
}

fn render_gaps_status_table(report: &NnsTopologyGapsReport) -> String {
    let headers = ["STATUS", "DETAIL"];
    let rows = [[report.status.clone(), "no topology join gaps".to_string()]];
    let alignments = [ColumnAlign::Left, ColumnAlign::Left];
    render_table(&headers, &rows, &alignments)
}

fn render_gaps_table(rows: &[NnsTopologyGapRow]) -> String {
    let headers = [
        "SUBJECT_KIND",
        "SUBJECT",
        "MISSING_RELATION",
        "REFERENCED_ID",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.subject_kind.clone(),
                row.subject.clone(),
                row.missing_relation.clone(),
                row.referenced_id.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}

fn render_capacity_summary_table(report: &NnsTopologyCapacityReport) -> String {
    let headers = ["FIELD", "VALUE"];
    let rows = [
        ["network".to_string(), report.network.clone()],
        ["status".to_string(), report.status.clone()],
        [
            "node_operators".to_string(),
            report.node_operator_count.to_string(),
        ],
        [
            "total_node_allowance".to_string(),
            report.total_node_allowance.to_string(),
        ],
        [
            "assigned_nodes".to_string(),
            report.assigned_node_count.to_string(),
        ],
        [
            "available_node_slots".to_string(),
            report.available_node_slots.to_string(),
        ],
        [
            "over_assigned_operators".to_string(),
            report.over_assigned_operator_count.to_string(),
        ],
        [
            "over_assigned_nodes".to_string(),
            report.over_assigned_node_count.to_string(),
        ],
        [
            "unknown_node_count_operators".to_string(),
            report.unknown_node_count_operator_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}

fn render_capacity_attention_table(report: &NnsTopologyCapacityReport) -> String {
    let attention_rows = report
        .capacity
        .iter()
        .filter(|row| matches!(row.status.as_str(), "over" | "unknown"))
        .collect::<Vec<_>>();
    if attention_rows.is_empty() {
        let headers = ["STATUS", "DETAIL"];
        let rows = [[
            report.status.clone(),
            "no capacity attention rows".to_string(),
        ]];
        let alignments = [ColumnAlign::Left, ColumnAlign::Left];
        return render_table(&headers, &rows, &alignments);
    }

    let headers = [
        "NODE_OPERATOR",
        "NODE_PROVIDER",
        "DATA_CENTER",
        "ALLOWANCE",
        "NODES",
        "AVAILABLE",
        "OVER",
        "UTILIZATION",
        "STATUS",
    ];
    let rows = attention_rows
        .iter()
        .map(|row| {
            [
                compact_text(&row.node_operator_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&row.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                row.data_center_id.clone(),
                row.node_allowance.to_string(),
                optional_u64_text(row.assigned_node_count),
                optional_u64_text(row.available_node_slots),
                optional_u64_text(row.over_assigned_node_count),
                row.utilization.clone(),
                row.status.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}

fn render_regions_table(rows: &[NnsTopologyRegionRow]) -> String {
    let headers = [
        "REGION",
        "DATA_CENTERS",
        "NODE_OPERATORS",
        "NODE_PROVIDERS",
        "NODES",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.region.clone(),
                row.data_center_count.to_string(),
                row.node_operator_count.to_string(),
                row.node_provider_count.to_string(),
                row.node_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}

fn render_providers_table(rows: &[NnsTopologyProviderRow]) -> String {
    let headers = [
        "NODE_PROVIDER",
        "STATUS",
        "GOV_NODES",
        "NODES",
        "OPERATORS",
        "DATA_CENTERS",
        "REGIONS",
        "ALLOWANCE",
        "AVAILABLE",
        "OVER",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                compact_text(&row.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                row.status.clone(),
                optional_u64_text(row.governance_node_count),
                row.topology_node_count.to_string(),
                row.node_operator_count.to_string(),
                row.data_center_count.to_string(),
                row.region_count.to_string(),
                row.total_node_allowance.to_string(),
                row.available_node_slots.to_string(),
                row.over_assigned_node_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn render_registry_version_table(rows: &[NnsTopologyRegistryVersionRow]) -> String {
    let headers = ["SOURCE", "VERSION", "FETCHED_AT", "STALE", "ENDPOINT"];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.source.clone(),
                row.registry_version.to_string(),
                row.fetched_at.clone(),
                row.stale
                    .map_or_else(|| "-".to_string(), |stale| yes_no(stale).to_string()),
                row.source_endpoint.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}

fn render_refresh_table(report: &NnsTopologyRefreshReport) -> String {
    let headers = [
        "SOURCE",
        "COUNT",
        "VERSION",
        "FETCHED_AT",
        "WROTE",
        "REPLACED",
        "CACHE",
    ];
    let rows = report
        .components
        .iter()
        .map(|row| {
            [
                row.source.clone(),
                row.item_count.to_string(),
                row.registry_version.to_string(),
                row.fetched_at.clone(),
                yes_no(row.wrote_cache).to_string(),
                yes_no(row.replaced_existing_cache).to_string(),
                row.cache_path.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}
