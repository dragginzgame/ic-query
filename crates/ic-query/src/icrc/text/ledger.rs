//! Module: icrc::text::ledger
//!
//! Responsibility: render ICRC ledger metadata, discovery, and evidence reports as text.
//! Does not own: transaction history, account views, live reads, or JSON output.
//! Boundary: formats token metadata and capability or certificate evidence for humans.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::push_table_section;
use crate::{
    human_quantity::byte_count_text,
    icrc::model::{
        IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcIndexReport, IcrcTipCertificateReport,
        IcrcTokenReport,
    },
    text_value::{optional_text, truncate_text},
    token_metadata_text::{
        render_metadata_rows_table, render_standard_rows_table, token_summary_lines,
    },
};

const ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT: usize = 160;
const ICRC_DETAIL_TEXT_LIMIT: usize = 160;

#[must_use]
pub fn icrc_token_report_text(report: &IcrcTokenReport) -> String {
    let mut lines = vec![format!("ledger_canister_id: {}", report.ledger_canister_id)];
    lines.extend(token_summary_lines(&report.token));
    lines.extend([
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]);
    push_table_section(
        &mut lines,
        &report.token.supported_standards,
        render_standard_rows_table,
    );
    push_table_section(&mut lines, &report.token.metadata, |rows| {
        render_metadata_rows_table(rows, report.token.decimals)
    });
    lines.join("\n")
}

#[must_use]
pub fn icrc_index_report_text(report: &IcrcIndexReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!(
            "index_canister_id: {}",
            optional_text(report.index_canister_id.as_ref())
        ),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ];
    if let Some(error) = report.index_error.as_deref() {
        lines.push(format!("index_error: {}", terminal_text(error)));
    }
    lines.join("\n")
}

#[must_use]
pub fn icrc_tip_certificate_report_text(report: &IcrcTipCertificateReport) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("certificate_present: {}", report.certificate_present),
        format!(
            "certificate_bytes: {}",
            optional_byte_count_text(report.certificate_bytes)
        ),
        format!(
            "hash_tree_bytes: {}",
            optional_byte_count_text(report.hash_tree_bytes)
        ),
        format!(
            "certificate_hex: {}",
            optional_truncated_text(
                report.certificate_hex.as_ref(),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
        format!(
            "hash_tree_hex: {}",
            optional_truncated_text(
                report.hash_tree_hex.as_ref(),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]
    .join("\n")
}

#[must_use]
pub fn icrc_capabilities_report_text(report: &IcrcCapabilitiesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("standard_count: {}", report.supported_standards.len()),
        format!("capability_count: {}", report.capabilities.len()),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ];
    push_table_section(
        &mut lines,
        &report.supported_standards,
        render_standard_rows_table,
    );
    push_table_section(
        &mut lines,
        &report.capabilities,
        render_capability_rows_table,
    );
    lines.join("\n")
}

fn render_capability_rows_table(rows: &[IcrcCapabilityRow]) -> String {
    render(
        &["CAPABILITY", "METHOD", "STATUS", "DETAIL"],
        &rows
            .iter()
            .map(|capability| {
                [
                    capability.capability.clone(),
                    capability.method.clone(),
                    capability.status.as_str().to_string(),
                    capability_detail_text(capability),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
        ],
        REPORT_TABLE_OPTIONS,
    )
}

fn optional_byte_count_text(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_string(), |value| byte_count_text(value as u128))
}

fn optional_truncated_text(value: Option<&String>, limit: usize) -> String {
    value.map_or_else(|| "-".to_string(), |value| truncate_text(value, limit))
}

fn capability_detail_text(row: &IcrcCapabilityRow) -> String {
    row.details.as_ref().or(row.error.as_ref()).map_or_else(
        || "-".to_string(),
        |detail| truncate_text(detail, ICRC_DETAIL_TEXT_LIMIT),
    )
}
