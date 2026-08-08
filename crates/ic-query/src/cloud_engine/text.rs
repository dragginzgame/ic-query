//! Module: cloud_engine::text
//!
//! Responsibility: render compact human-facing CloudEngine reports.
//! Does not own: report construction, JSON output, live calls, or process output.
//! Boundary: formats cycle amounts only for text while JSON retains raw decimal fields.

use super::{CloudEngineOperatorReport, CloudEnginePricesReport, CloudEngineReportContext};
use crate::{
    human_quantity::decimal_cycle_count_text,
    table::{ColumnAlign, render_table},
    text_value::{optional_text, sanitize_text, yes_no},
};

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
