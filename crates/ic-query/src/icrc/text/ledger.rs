//! Module: icrc::text::ledger
//!
//! Responsibility: render ICRC ledger metadata, discovery, and evidence reports as text.
//! Does not own: transaction history, account views, live reads, or JSON output.
//! Boundary: formats token metadata and capability or certificate evidence for humans.

use super::{LEFT_2_ALIGNMENTS, push_table_section};
use crate::{
    icrc::model::{
        IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcIndexReport, IcrcTipCertificateReport,
        IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenStandardRow,
    },
    table::{ColumnAlign, render_table},
    text_value::{optional_text, sanitize_text, truncate_text},
    token_amount::base_units_decimal_text,
    token_metadata_text::token_metadata_value_text as metadata_value_text,
};

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;
const ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT: usize = 160;
const ICRC_DETAIL_TEXT_LIMIT: usize = 160;
const STANDARD_TABLE_HEADERS: [&str; 2] = ["STANDARD", "URL"];

#[must_use]
pub fn icrc_token_report_text(report: &IcrcTokenReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("token_name: {}", sanitize_text(&report.token_name)),
        format!("token_symbol: {}", sanitize_text(&report.token_symbol)),
        format!("decimals: {}", report.decimals),
        format!(
            "transfer_fee: {}",
            base_units_decimal_text(&report.transfer_fee, report.decimals)
        ),
        format!(
            "total_supply: {}",
            base_units_decimal_text(&report.total_supply, report.decimals)
        ),
        format!(
            "minting_account_owner: {}",
            optional_text(report.minting_account_owner.as_ref())
        ),
        format!(
            "minting_account_subaccount_hex: {}",
            optional_text(report.minting_account_subaccount_hex.as_ref())
        ),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ];
    push_table_section(
        &mut lines,
        &report.supported_standards,
        render_standard_rows_table,
    );
    push_table_section(&mut lines, &report.metadata, |rows| {
        render_metadata_rows_table(rows, report.decimals)
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
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ];
    if let Some(error) = report.index_error.as_deref() {
        lines.push(format!("index_error: {}", sanitize_text(error)));
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
            optional_usize_text(report.certificate_bytes)
        ),
        format!(
            "hash_tree_bytes: {}",
            optional_usize_text(report.hash_tree_bytes)
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
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
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
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
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

fn render_standard_rows_table(standards: &[IcrcTokenStandardRow]) -> String {
    render_table(
        &STANDARD_TABLE_HEADERS,
        &standards
            .iter()
            .map(|standard| [standard.name.clone(), standard.url.clone()])
            .collect::<Vec<_>>(),
        &LEFT_2_ALIGNMENTS,
    )
}

fn render_metadata_rows_table(rows: &[IcrcTokenMetadataRow], decimals: u8) -> String {
    render_table(
        &["METADATA", "TYPE", "VALUE"],
        &rows
            .iter()
            .map(|row| {
                [
                    row.key.clone(),
                    row.value_type.as_str().to_string(),
                    truncate_text(
                        &metadata_value_text(&row.key, &row.value, decimals),
                        ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
    )
}

fn render_capability_rows_table(rows: &[IcrcCapabilityRow]) -> String {
    render_table(
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
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
        ],
    )
}

fn optional_usize_text(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_truncated_text(value: Option<&String>, limit: usize) -> String {
    value.map_or_else(|| "-".to_string(), |value| truncate_text(value, limit))
}

fn capability_detail_text(row: &IcrcCapabilityRow) -> String {
    let detail = row.details.as_ref().or(row.error.as_ref());
    optional_truncated_text(detail, ICRC_DETAIL_TEXT_LIMIT)
}
