mod delay;
mod economic;
mod limits;
mod permissions;
mod rewards;
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
    [
        economic::rows(parameters),
        delay::rows(parameters),
        limits::rows(parameters),
        permissions::rows(parameters),
        rewards::rows(parameters),
    ]
    .concat()
}
