//! Module: cloud_engine::text
//!
//! Responsibility: render compact human-facing CloudEngine reports.
//! Does not own: report construction, JSON output, live calls, or process output.
//! Boundary: formats cycle amounts only for text while JSON retains raw decimal fields.

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
use super::{CloudEngineListReport, CloudEngineOperatorLookupStatus};
use super::{CloudEngineOperatorReport, CloudEnginePricesReport, CloudEngineReportContext};
use crate::{
    human_quantity::decimal_cycle_count_text,
    table::{ColumnAlign, render_table},
    text_value::{optional_text, sanitize_text, yes_no},
};

/// Render the Registry CloudEngine inventory and separate operator-binding observations.
#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
#[must_use]
pub fn cloud_engine_list_report_text(report: &CloudEngineListReport) -> String {
    let mut lines = list_context_lines(report);
    lines.push(String::new());
    lines.push("CloudEngine subnets".to_string());
    if report.cloud_engines.is_empty() {
        lines.push("none".to_string());
        return lines.join("\n");
    }
    lines.push(list_table(report));
    append_lookup_failures(report, &mut lines);
    lines.join("\n")
}

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
fn list_context_lines(report: &CloudEngineListReport) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!(
            "registry_authority: {}",
            sanitize_text(&report.registry_authority)
        ),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("registry_assurance: {}", report.registry_assurance.as_str()),
        format!(
            "registry_source_endpoints: {}",
            report
                .registry_source_endpoints
                .iter()
                .map(|endpoint| sanitize_text(endpoint))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "catalog_fetched_at: {}",
            sanitize_text(&report.catalog_fetched_at)
        ),
        format!(
            "catalog_cache_disposition: {}",
            report.catalog_cache_disposition.as_str()
        ),
        format!("catalog_stale: {}", yes_no(report.catalog_stale)),
        String::new(),
        format!(
            "control_plane_authority: {}",
            sanitize_text(&report.control_plane_authority)
        ),
        format!(
            "control_plane_canister_id: {}",
            report.control_plane_canister_id
        ),
        format!(
            "control_plane_source_endpoint: {}",
            sanitize_text(&report.control_plane_source_endpoint)
        ),
        format!(
            "control_plane_fetched_at: {}",
            sanitize_text(&report.control_plane_fetched_at)
        ),
        format!(
            "control_plane_certified: {}",
            yes_no(report.control_plane_certified)
        ),
        format!(
            "control_plane_point_in_time_guaranteed: {}",
            yes_no(report.control_plane_point_in_time_guaranteed)
        ),
        format!(
            "control_plane_lookup_attempt_count: {}",
            report.control_plane_lookup_attempt_count
        ),
        format!(
            "operator_bindings: {} resolved, {} absent, {} failed",
            report.operator_binding_count,
            report.missing_operator_binding_count,
            report.operator_lookup_failure_count
        ),
    ]
}

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
fn list_table(report: &CloudEngineListReport) -> String {
    let headers = ["Label", "Subnet", "Nodes", "Binding", "Operator"];
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    let rows = report
        .cloud_engines
        .iter()
        .map(|row| {
            [
                sanitize_text(&row.subnet_label),
                row.subnet_id.clone(),
                row.node_count
                    .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
                row.operator_lookup_status.as_str().to_string(),
                row.operator_canister_id
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect::<Vec<_>>();
    render_table(&headers, &rows, &alignments)
}

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
fn append_lookup_failures(report: &CloudEngineListReport, lines: &mut Vec<String>) {
    let failures = report
        .cloud_engines
        .iter()
        .filter(|row| row.operator_lookup_status == CloudEngineOperatorLookupStatus::Failed)
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        lines.push(String::new());
        lines.push("Operator lookup failures".to_string());
        lines.extend(failures.into_iter().map(|row| {
            format!(
                "  {}: {}",
                row.subnet_id,
                sanitize_text(
                    row.operator_lookup_error
                        .as_deref()
                        .unwrap_or("unspecified lookup failure")
                )
            )
        }));
    }
}

/// Render one CloudEngine Subnet-to-operator report.
#[must_use]
pub fn cloud_engine_operator_report_text(report: &CloudEngineOperatorReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!("subnet_id: {}", report.subnet_id),
        format!(
            "operator_binding_present: {}",
            yes_no(report.operator_binding_present)
        ),
        format!(
            "operator_canister_id: {}",
            optional_text(report.operator_canister_id.as_ref())
        ),
        format!(
            "engine_owner: {}",
            optional_text(report.engine_owner.as_ref())
        ),
        format!(
            "platform_admin: {}",
            optional_text(report.platform_admin.as_ref())
        ),
        format!(
            "caffeine_enabled: {}",
            report
                .caffeine_enabled
                .map_or_else(|| "-".to_string(), |enabled| yes_no(enabled).to_string())
        ),
        format!(
            "claimed_domain_count: {}",
            report
                .claimed_domain_count
                .map_or_else(|| "-".to_string(), |count| count.to_string())
        ),
    ]);

    if let Some(domains) = report.claimed_domains.as_ref()
        && !domains.is_empty()
    {
        lines.push(String::new());
        lines.push("Claimed domains".to_string());
        lines.extend(
            domains
                .iter()
                .map(|domain| format!("  {}", sanitize_text(domain))),
        );
    }
    lines.join("\n")
}

/// Render one bounded CloudEngine marketplace price report.
#[must_use]
pub fn cloud_engine_prices_report_text(report: &CloudEnginePricesReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!("network_fee: {}", report.network_fee),
        format!("price_count: {}", report.price_count),
    ]);

    if !report.prices.is_empty() {
        let headers = [
            "Key",
            "Node type",
            "Data center",
            "Provider",
            "Net/month",
            "Gross/month",
            "Updated (ns)",
        ];
        let alignments = [
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Right,
            ColumnAlign::Right,
        ];
        let rows = report
            .prices
            .iter()
            .map(|row| {
                [
                    row.key.clone(),
                    row.node_type.to_string(),
                    row.data_center_id
                        .as_deref()
                        .map_or_else(|| "-".to_string(), sanitize_text),
                    row.provider_id.clone().unwrap_or_else(|| "-".to_string()),
                    decimal_cycle_count_text(&row.net_cycles_per_month),
                    decimal_cycle_count_text(&row.gross_cycles_per_month),
                    row.updated_at_unix_nanos.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(String::new());
        lines.push("Marketplace prices".to_string());
        lines.push(render_table(&headers, &rows, &alignments));
    }
    lines.join("\n")
}

fn context_lines(context: &CloudEngineReportContext) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&context.network)),
        format!("authority: {}", sanitize_text(&context.authority)),
        format!("engine_canister_id: {}", context.engine_canister_id),
        format!("fetched_at: {}", sanitize_text(&context.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&context.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&context.fetched_by)),
        format!("certified: {}", yes_no(context.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(context.point_in_time_guaranteed)
        ),
        format!("query_call_count: {}", context.query_call_count),
    ]
}
