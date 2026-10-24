//! Module: sns::report::text::token
//!
//! Responsibility: render SNS token reports as text.
//! Does not own: token source reads, report construction, or JSON output.
//! Boundary: formats token metadata and base-unit amounts for humans.

use crate::{
    sns::report::SnsTokenReport,
    text_value::optional_text,
    token_metadata_text::{
        render_metadata_rows_table, render_standard_rows_table, token_summary_lines,
    },
};
use ic_host_text::escape::terminal_text;

#[must_use]
pub fn sns_token_report_text(report: &SnsTokenReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("sns_index_canister_id: {}", report.sns_index_canister_id),
        format!(
            "ledger_index_canister_id: {}",
            optional_text(report.ledger_index_canister_id.as_ref())
        ),
    ];
    lines.extend(token_summary_lines(&report.token));
    lines.extend([
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]);
    if let Some(error) = report.ledger_index_error.as_deref() {
        lines.push(format!("ledger_index_error: {}", terminal_text(error)));
    }
    if !report.token.supported_standards.is_empty() {
        lines.push(String::new());
        lines.push(render_standard_rows_table(
            &report.token.supported_standards,
        ));
    }
    if !report.token.metadata.is_empty() {
        lines.push(String::new());
        lines.push(render_metadata_rows_table(
            &report.token.metadata,
            report.token.decimals,
        ));
    }
    lines.join("\n")
}
