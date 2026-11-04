//! Module: sns::token
//!
//! Responsibility: render SNS token reports as text.
//! Does not own: token source reads, report construction, or JSON output.
//! Boundary: formats token metadata and base-unit amounts for humans.

use crate::text_value::field;
use crate::token_metadata_text::append_token_metadata_tables;
use crate::token_metadata_text::token_summary_lines;
use ic_query::sns::SnsTokenReport;

#[must_use]
/// Render the SNS token report as human-facing text.
pub fn sns_token_report_text(report: &SnsTokenReport) -> String {
    let mut lines = vec![
        field("network", &report.context.network),
        field("sns_id", &(report.context.id)),
        field("name", &report.context.name),
        field("root_canister_id", &(report.context.root_canister_id)),
        field("ledger_canister_id", &(report.ledger_canister_id)),
        field("sns_index_canister_id", &(report.sns_index_canister_id)),
        field(
            "ledger_index_canister_id",
            &(report.ledger_index.canister_id().unwrap_or("-")),
        ),
    ];
    lines.extend(token_summary_lines(&report.token));
    lines.extend([
        field(
            "sns_wasm_canister_id",
            &(report.context.sns_wasm_canister_id),
        ),
        field("fetched_at", &report.context.fetched_at),
        field("source_endpoint", &report.context.source_endpoint),
    ]);
    if let Some(error) = report.ledger_index.unavailable_reason() {
        lines.push(field("ledger_index_error", &(error)));
    }
    append_token_metadata_tables(&mut lines, &report.token);
    lines.join("\n")
}
