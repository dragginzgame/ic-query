//! Module: sns::report::text::params
//!
//! Responsibility: render SNS governance parameter reports as text.
//! Does not own: parameter fetching, report construction, or JSON output.
//! Boundary: groups governance parameter rows into a human-readable table.

mod rows;

use crate::{
    sns::report::{SnsGovernanceParameters, SnsParamsReport},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn sns_params_report_text(report: &SnsParamsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    lines.push(String::new());
    lines.push(render_table(
        &["PARAMETER", "VALUE"],
        &sns_params_text_rows(&report.parameters),
        &[ColumnAlign::Left, ColumnAlign::Right],
    ));
    lines.join("\n")
}

fn sns_params_text_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    rows::sns_params_text_rows(parameters)
}
