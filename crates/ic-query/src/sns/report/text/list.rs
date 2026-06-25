//! Module: sns::report::text::list
//!
//! Responsibility: render deployed SNS list reports as text.
//! Does not own: report construction, list sorting, lookup ids, or JSON output.
//! Boundary: formats list DTO rows into compact or verbose human-readable tables.

use crate::sns::report::{SnsListReport, short_principal};
use crate::table::{ColumnAlign, render_table};

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("network: {}", report.network));
    lines.push(format!(
        "sns_wasm_canister_id: {}",
        report.sns_wasm_canister_id
    ));
    lines.push(format!("sns_count: {}", report.sns_count));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("sort: {}", report.sort));
    lines.push(format!("metadata_errors: {}", report.metadata_error_count));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "ID",
                "NAME",
                "ROOT",
                "GOVERNANCE",
                "LEDGER",
                "SWAP",
                "INDEX",
            ],
            &report
                .sns_instances
                .iter()
                .map(|sns| {
                    [
                        sns.id.to_string(),
                        sns.name.clone(),
                        principal_for_list(&sns.root_canister_id, report.verbose),
                        principal_for_list(&sns.governance_canister_id, report.verbose),
                        principal_for_list(&sns.ledger_canister_id, report.verbose),
                        principal_for_list(&sns.swap_canister_id, report.verbose),
                        principal_for_list(&sns.index_canister_id, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && report.metadata_error_count > 0 {
        lines.push(String::new());
        lines.push("metadata_error_details:".to_string());
        for (governance_canister_id, error) in report.sns_instances.iter().filter_map(|sns| {
            sns.metadata_error
                .as_deref()
                .map(|error| (&sns.governance_canister_id, error))
        }) {
            lines.push(format!("- {governance_canister_id}: {error}"));
        }
    }
    lines.join("\n")
}

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}
