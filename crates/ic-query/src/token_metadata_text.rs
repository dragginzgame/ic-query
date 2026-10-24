//! Module: token_metadata_text
//!
//! Responsibility: shared text formatting for ICRC-style token metadata.
//! Does not own: report DTOs, family-specific provenance, or JSON output.
//! Boundary: formats raw metadata values for human text.

use crate::{
    icrc::{IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow},
    text_value::{REPORT_TABLE_OPTIONS, optional_text, truncate_text},
    token_amount::base_units_decimal_text,
};
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};
use serde_json::Value as JsonValue;

const ICRC_FEE_METADATA_KEY: &str = "icrc1:fee";
const ICRC_LOGO_METADATA_KEY: &str = "icrc1:logo";

pub fn token_metadata_value_text(key: &str, value: &JsonValue, decimals: u8) -> String {
    if key == ICRC_LOGO_METADATA_KEY {
        return if metadata_value_is_present(value) {
            "present".to_string()
        } else {
            "-".to_string()
        };
    }
    let value = metadata_value_text(value);
    if key == ICRC_FEE_METADATA_KEY {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}

fn metadata_value_is_present(value: &JsonValue) -> bool {
    match value {
        JsonValue::String(value) => !value.trim().is_empty(),
        JsonValue::Array(value) => !value.is_empty(),
        JsonValue::Object(value) => !value.is_empty(),
        JsonValue::Bool(value) => *value,
        JsonValue::Number(_) => true,
        JsonValue::Null => false,
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

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;

pub fn token_summary_lines(token: &IcrcTokenData) -> Vec<String> {
    vec![
        format!("token_name: {}", terminal_text(&token.token_name)),
        format!("token_symbol: {}", terminal_text(&token.token_symbol)),
        format!("decimals: {}", token.decimals),
        format!(
            "transfer_fee: {}",
            base_units_decimal_text(&token.transfer_fee, token.decimals)
        ),
        format!(
            "total_supply: {}",
            base_units_decimal_text(&token.total_supply, token.decimals)
        ),
        format!(
            "minting_account_owner: {}",
            optional_text(token.minting_account_owner.as_ref())
        ),
        format!(
            "minting_account_subaccount_hex: {}",
            optional_text(token.minting_account_subaccount_hex.as_ref())
        ),
    ]
}

pub fn render_standard_rows_table(standards: &[IcrcTokenStandardRow]) -> String {
    render(
        &["STANDARD", "URL"],
        &standards
            .iter()
            .map(|standard| [standard.name.clone(), standard.url.clone()])
            .collect::<Vec<_>>(),
        &[Alignment::Left, Alignment::Left],
        REPORT_TABLE_OPTIONS,
    )
}

pub fn render_metadata_rows_table(rows: &[IcrcTokenMetadataRow], decimals: u8) -> String {
    render(
        &["METADATA", "TYPE", "VALUE"],
        &rows
            .iter()
            .map(|row| {
                [
                    row.key.clone(),
                    row.value_type.as_str().to_string(),
                    truncate_text(
                        &token_metadata_value_text(&row.key, &row.value, decimals),
                        ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        &[Alignment::Left, Alignment::Left, Alignment::Left],
        REPORT_TABLE_OPTIONS,
    )
}
