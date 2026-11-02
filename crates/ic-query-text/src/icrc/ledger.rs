//! Module: icrc::ledger
//!
//! Responsibility: render ICRC ledger metadata, discovery, and evidence reports as text.
//! Does not own: transaction history, account views, live reads, or JSON output.
//! Boundary: formats token metadata and capability or certificate evidence for humans.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use crate::human_quantity::byte_count_text;
use crate::icrc::push_table_section;
use crate::text_value::truncate_text;
use crate::token_metadata_text::append_token_metadata_tables;
use crate::token_metadata_text::render_standard_rows_table;
use crate::token_metadata_text::token_summary_lines;
use ic_query::icrc::IcrcCapabilitiesReport;
use ic_query::icrc::IcrcCapabilityRow;
use ic_query::icrc::IcrcIndexReport;
use ic_query::icrc::IcrcTipCertificateReport;
use ic_query::icrc::IcrcTokenReport;

const ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT: usize = 160;
const ICRC_DETAIL_TEXT_LIMIT: usize = 160;

#[must_use]
/// Render the ICRC token report as human-facing text.
pub fn icrc_token_report_text(report: &IcrcTokenReport) -> String {
    let mut lines = vec![format!(
        "ledger_canister_id: {}",
        report.provenance.ledger_canister_id
    )];
    lines.extend(token_summary_lines(&report.token));
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    append_token_metadata_tables(&mut lines, &report.token);
    lines.join("\n")
}

#[must_use]
/// Render the ICRC index report as human-facing text.
pub fn icrc_index_report_text(report: &IcrcIndexReport) -> String {
    let mut lines = vec![
        format!(
            "ledger_canister_id: {}",
            report.provenance.ledger_canister_id
        ),
        format!(
            "index_canister_id: {}",
            terminal_text(report.index.canister_id().unwrap_or("-"))
        ),
    ];
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    if let Some(error) = report.index.unavailable_reason() {
        lines.push(format!("index_error: {}", terminal_text(error)));
    }
    lines.join("\n")
}

#[must_use]
/// Render the ICRC tip certificate report as human-facing text.
pub fn icrc_tip_certificate_report_text(report: &IcrcTipCertificateReport) -> String {
    let mut lines = vec![
        format!(
            "ledger_canister_id: {}",
            report.provenance.ledger_canister_id
        ),
        format!("certificate_present: {}", report.certificate.is_some()),
        format!(
            "certificate_bytes: {}",
            optional_byte_count_text(
                report
                    .certificate
                    .as_ref()
                    .map(|evidence| evidence.certificate_bytes)
            )
        ),
        format!(
            "hash_tree_bytes: {}",
            optional_byte_count_text(
                report
                    .certificate
                    .as_ref()
                    .map(|evidence| evidence.hash_tree_bytes)
            )
        ),
        format!(
            "certificate_hex: {}",
            optional_truncated_text(
                report
                    .certificate
                    .as_ref()
                    .map(|evidence| &evidence.certificate_hex),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
        format!(
            "hash_tree_hex: {}",
            optional_truncated_text(
                report
                    .certificate
                    .as_ref()
                    .map(|evidence| &evidence.hash_tree_hex),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
    ];
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    lines.join("\n")
}

#[must_use]
/// Render the ICRC capabilities report as human-facing text.
pub fn icrc_capabilities_report_text(report: &IcrcCapabilitiesReport) -> String {
    let mut lines = vec![
        format!(
            "ledger_canister_id: {}",
            report.provenance.ledger_canister_id
        ),
        format!("standard_count: {}", report.supported_standards.len()),
        format!("capability_count: {}", report.capabilities.len()),
    ];
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
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
