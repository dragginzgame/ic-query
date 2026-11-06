//! Module: cloud_engine
//!
//! Responsibility: render compact human-facing CloudEngine reports.
//! Does not own: report construction, JSON output, live calls, or process output.
//! Boundary: formats cycle amounts only for text while JSON retains raw decimal fields.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::human_quantity::decimal_cycle_count_text;
use crate::text_value::optional_text;
use crate::text_value::yes_no;
#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
use ic_query::cloud_engine::CloudEngineListReport;
use ic_query::cloud_engine::CloudEngineOperatorReport;
use ic_query::cloud_engine::CloudEnginePricesReport;
use ic_query::cloud_engine::CloudEngineReportContext;

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
        field("network", &report.network),
        field("registry_authority", &report.registry_authority),
        field("registry_canister_id", &(report.registry_canister_id)),
        field("registry_version", &(report.registry_version)),
        field("registry_assurance", &(report.registry_assurance.as_str())),
        field(
            "registry_source_endpoints",
            &(report
                .registry_source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(", ")),
        ),
        field("catalog_fetched_at", &report.catalog_fetched_at),
        field(
            "catalog_cache_disposition",
            &(report.catalog_cache_disposition.as_str()),
        ),
        field("catalog_stale", &(yes_no(report.catalog_stale))),
        String::new(),
        field("control_plane_authority", &report.control_plane_authority),
        field(
            "control_plane_canister_id",
            &(report.control_plane_canister_id),
        ),
        field(
            "control_plane_source_endpoint",
            &report.control_plane_source_endpoint,
        ),
        field("control_plane_fetched_at", &report.control_plane_fetched_at),
        field(
            "control_plane_certified",
            &(yes_no(report.control_plane_certified)),
        ),
        field(
            "control_plane_point_in_time_guaranteed",
            &(yes_no(report.control_plane_point_in_time_guaranteed)),
        ),
        field(
            "control_plane_lookup_attempt_count",
            &(report.control_plane_lookup_attempt_count),
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
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Left,
    ];
    let rows = report
        .cloud_engines
        .iter()
        .map(|row| {
            [
                terminal_text(&row.subnet_label).into_owned(),
                row.subnet_id.clone(),
                row.node_count
                    .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
                match &row.operator {
                    Ok(Some(_)) => "resolved",
                    Ok(None) => "absent",
                    Err(_) => "failed",
                }
                .to_string(),
                row.operator
                    .as_ref()
                    .ok()
                    .and_then(Option::as_ref)
                    .map_or_else(|| "-".to_string(), ToString::to_string),
            ]
        })
        .collect::<Vec<_>>();
    table(&headers, &rows, &alignments)
}

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
fn append_lookup_failures(report: &CloudEngineListReport, lines: &mut Vec<String>) {
    let failures = report
        .cloud_engines
        .iter()
        .filter_map(|row| row.operator.as_ref().err().map(|error| (row, error)))
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        lines.push(String::new());
        lines.push("Operator lookup failures".to_string());
        lines.extend(
            failures
                .into_iter()
                .map(|(row, error)| format!("  {}: {}", row.subnet_id, terminal_text(error))),
        );
    }
}

/// Render one CloudEngine Subnet-to-operator report.
#[must_use]
pub fn cloud_engine_operator_report_text(report: &CloudEngineOperatorReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        subnet_id,
        operator_binding_present = yes_no(report.operator_canister_id.is_some()),
        operator_canister_id = optional_text(report.operator_canister_id.as_ref()),
        engine_owner = optional_text(report.engine_owner.as_ref()),
        platform_admin = optional_text(report.platform_admin.as_ref()),
        caffeine_enabled = report
                .caffeine_enabled
                .map_or_else(|| "-".to_string(), |enabled| yes_no(enabled).to_string()),
        claimed_domain_count = report
                .claimed_domains
                .as_ref()
                .map(Vec::len)
                .map_or_else(|| "-".to_string(), |count| count.to_string()),
    ));

    if let Some(domains) = report.claimed_domains.as_ref()
        && !domains.is_empty()
    {
        lines.push(String::new());
        lines.push("Claimed domains".to_string());
        lines.extend(
            domains
                .iter()
                .map(|domain| format!("  {}", terminal_text(domain))),
        );
    }
    lines.join("\n")
}

/// Render one bounded CloudEngine marketplace price report.
#[must_use]
pub fn cloud_engine_prices_report_text(report: &CloudEnginePricesReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        network_fee,
        price_count = report.prices.len(),
    ));

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
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ];
        let rows = report
            .prices
            .iter()
            .map(|row| {
                [
                    row.key.clone(),
                    row.node_type.to_string(),
                    row.data_center_id.as_deref().map_or_else(
                        || "-".to_string(),
                        |value| terminal_text(value).into_owned(),
                    ),
                    row.provider_id.clone().unwrap_or_else(|| "-".to_string()),
                    decimal_cycle_count_text(&row.net_cycles_per_month),
                    decimal_cycle_count_text(&row.gross_cycles_per_month),
                    row.updated_at_unix_nanos.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(String::new());
        lines.push("Marketplace prices".to_string());
        lines.push(table(&headers, &rows, &alignments));
    }
    lines.join("\n")
}

fn context_lines(context: &CloudEngineReportContext) -> Vec<String> {
    Vec::from(crate::text_value::report_fields!(context;
        network,
        authority,
        engine_canister_id,
        fetched_at,
        source_endpoint,
        fetched_by,
        certified = yes_no(context.certified),
        point_in_time_guaranteed = yes_no(context.point_in_time_guaranteed),
        query_call_count,
    ))
}

mod provider;
pub use provider::*;

mod node;
pub use node::*;
