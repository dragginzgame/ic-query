//! Module: sns::report::text::list
//!
//! Responsibility: render deployed SNS list reports as text.
//! Does not own: report construction, list sorting, lookup ids, or JSON output.
//! Boundary: formats list DTO rows into compact or verbose human-readable tables.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use crate::sns::report::{SnsListReport, short_principal};

use crate::text_value::yes_no;

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("network: {}", terminal_text(&report.network)));
    lines.push(format!(
        "sns_wasm_canister_id: {}",
        report.sns_wasm_canister_id
    ));
    lines.push(format!("all_lifecycles: {}", yes_no(report.all_lifecycles)));
    lines.push(format!("catalog_sns_count: {}", report.catalog_sns_count));
    lines.push(format!("excluded_sns_count: {}", report.excluded_sns_count));
    lines.push(format!("sns_count: {}", report.sns_count));
    lines.push(format!("fetched_at: {}", terminal_text(&report.fetched_at)));
    lines.push(format!("data_source: {}", report.data_source));
    if let Some(cache_path) = report.cache_path.as_ref() {
        lines.push(format!("cache_path: {}", terminal_text(cache_path)));
    }
    if let Some(cache_complete) = report.cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
    lines.push(format!(
        "source_endpoint: {}",
        terminal_text(&report.source_endpoint)
    ));
    lines.push(format!("sort: {}", report.sort));
    lines.push(format!("metadata_errors: {}", report.metadata_error_count));
    lines.push(format!(
        "lifecycle_errors: {}",
        report.lifecycle_error_count
    ));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        lines.push(render(
            &[
                "ID",
                "NAME",
                "ROOT",
                "GOVERNANCE",
                "LEDGER",
                "SWAP",
                "INDEX",
                "LIFECYCLE",
                "METADATA",
            ],
            &report
                .sns_instances
                .iter()
                .map(|sns| {
                    [
                        sns.id.to_string(),
                        sns.name.clone(),
                        principal_for_list(&sns.canisters.root_canister_id, report.verbose),
                        principal_for_list(&sns.canisters.governance_canister_id, report.verbose),
                        principal_for_list(&sns.canisters.ledger_canister_id, report.verbose),
                        principal_for_list(&sns.canisters.swap_canister_id, report.verbose),
                        principal_for_list(&sns.canisters.index_canister_id, report.verbose),
                        lifecycle_status(
                            sns.lifecycle_name.as_deref(),
                            sns.lifecycle_error.as_deref(),
                        )
                        .to_string(),
                        metadata_status(sns.metadata_error.as_deref()).to_string(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
        ));
    }
    lines.join("\n")
}

const fn lifecycle_status<'a>(name: Option<&'a str>, error: Option<&str>) -> &'a str {
    match (name, error) {
        (Some(name), None) => name,
        (None, Some(_)) => "error",
        _ => "-",
    }
}

fn metadata_status(error: Option<&str>) -> &'static str {
    match error {
        None => "ok",
        Some(error) if error.contains("no Wasm module") => "no_wasm",
        Some(_) => "error",
    }
}

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}
