//! Module: sns::report::text::token
//!
//! Responsibility: render SNS token reports as text.
//! Does not own: token source reads, report construction, or JSON output.
//! Boundary: formats token metadata and base-unit amounts for humans.

use super::common::{optional_text, token_metadata_value_text, truncate_text_value};
use crate::{
    sns::report::SnsTokenReport,
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
};

const SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;

#[must_use]
pub fn sns_token_report_text(report: &SnsTokenReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("sns_index_canister_id: {}", report.sns_index_canister_id),
        format!(
            "ledger_index_canister_id: {}",
            optional_text(report.ledger_index_canister_id.as_ref())
        ),
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
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.ledger_index_error.as_deref() {
        lines.push(format!("ledger_index_error: {error}"));
    }
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
                            SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}
