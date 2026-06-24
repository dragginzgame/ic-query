//! Module: icrc::text
//!
//! Responsibility: render generic ICRC reports as text.
//! Does not own: live source reads, JSON output, or command parsing.
//! Boundary: formats token metadata and base-unit amounts for humans.

use crate::{
    icrc::model::{IcrcBalanceReport, IcrcTokenMetadataRow, IcrcTokenReport},
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
};
use serde_json::Value as JsonValue;

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;

#[must_use]
pub(in crate::icrc) fn icrc_token_report_text(report: &IcrcTokenReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("token_name: {}", report.token_name),
        format!("token_symbol: {}", report.token_symbol),
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
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.supported_standards.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["STANDARD", "URL"],
            &report
                .supported_standards
                .iter()
                .map(|standard| [standard.name.clone(), standard.url.clone()])
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    if !report.metadata.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["METADATA", "TYPE", "VALUE"],
            &report
                .metadata
                .iter()
                .map(|row| {
                    [
                        row.key.clone(),
                        row.value_type.clone(),
                        truncate_text_value(
                            &token_metadata_value_text(row, report.decimals),
                            ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_balance_report_text(report: &IcrcBalanceReport) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!("token_symbol: {}", report.token_symbol),
        format!("decimals: {}", report.decimals),
        format!(
            "balance: {} {}",
            base_units_decimal_text(&report.balance, report.decimals),
            report.token_symbol
        ),
        format!("balance_base_units: {}", report.balance),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn token_metadata_value_text(row: &IcrcTokenMetadataRow, decimals: u8) -> String {
    let value = metadata_value_text(&row.value);
    if row.key == "icrc1:fee" {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}

fn metadata_value_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null => "-".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}
