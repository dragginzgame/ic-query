//! Module: token_metadata_text
//!
//! Responsibility: shared text formatting for ICRC-style token metadata.
//! Does not own: report DTOs, family-specific provenance, or JSON output.
//! Boundary: formats raw metadata values for human text.

use crate::text_value::optional_text;
use crate::text_value::table;
use crate::text_value::truncate_text;
use crate::text_value::{json_scalar_text, report_fields};
use crate::token_amount::base_units_decimal_text;
use ic_host_text::table::Alignment;
use ic_query::icrc::IcrcTokenData;
use ic_query::icrc::IcrcTokenMetadataRow;
use ic_query::icrc::IcrcTokenStandardRow;
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
    let value = json_scalar_text(value);
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

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;

pub fn token_summary_lines(token: &IcrcTokenData) -> Vec<String> {
    Vec::from(report_fields!(token;
        token_name,
        token_symbol,
        decimals,
        transfer_fee = base_units_decimal_text(&token.transfer_fee, token.decimals),
        total_supply = base_units_decimal_text(&token.total_supply, token.decimals),
        minting_account_owner = optional_text(token.minting_account_owner.as_ref()),
        minting_account_subaccount_hex = optional_text(token.minting_account_subaccount_hex.as_ref()),
    ))
}

pub fn append_token_metadata_tables(lines: &mut Vec<String>, token: &IcrcTokenData) {
    if !token.supported_standards.is_empty() {
        lines.push(String::new());
        lines.push(render_standard_rows_table(&token.supported_standards));
    }
    if !token.metadata.is_empty() {
        lines.push(String::new());
        lines.push(render_metadata_rows_table(&token.metadata, token.decimals));
    }
}

pub fn render_standard_rows_table(standards: &[IcrcTokenStandardRow]) -> String {
    table(
        &["STANDARD", "URL"],
        &standards
            .iter()
            .map(|standard| [standard.name.clone(), standard.url.clone()])
            .collect::<Vec<_>>(),
        &[Alignment::Left; 2],
    )
}

fn render_metadata_rows_table(rows: &[IcrcTokenMetadataRow], decimals: u8) -> String {
    table(
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
        &[Alignment::Left; 3],
    )
}
