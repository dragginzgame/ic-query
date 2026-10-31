//! Module: sns::report::text::list
//!
//! Responsibility: render deployed SNS list reports as text.
//! Does not own: report construction, list sorting, lookup ids, or JSON output.
//! Boundary: formats list DTO rows into compact or verbose human-readable tables.

use crate::sns::report::sns_swap_lifecycle_name;
use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use crate::sns::report::{SnsListReport, short_principal};

use crate::text_value::yes_no;

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    sns_list_report_text_with_metadata_style(report, |status, _| status.to_string())
}

/// Render an SNS list with caller-selected styling for its metadata status cells.
///
/// The callback receives the status label and whether metadata collection failed.
/// It may add trusted styling, such as ANSI colours, after escaping and alignment.
/// The caller owns terminal policy; this renderer never selects an output sink.
#[must_use]
pub fn sns_list_report_text_with_metadata_style(
    report: &SnsListReport,
    mut style: impl FnMut(&str, bool) -> String,
) -> String {
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
        let table = render(
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
                            sns_swap_lifecycle_name(sns.lifecycle),
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
        );
        // Host escapes cell newlines, so each data line corresponds to one known row.
        lines.push(
            table
                .lines()
                .take(2)
                .map(str::to_string)
                .chain(
                    table
                        .lines()
                        .skip(2)
                        .zip(&report.sns_instances)
                        .map(|(line, sns)| {
                            let failed = sns.metadata_error.is_some();
                            let status = metadata_status(sns.metadata_error.as_deref());
                            let prefix = &line[..line.len() - status.len()];
                            format!("{prefix}{}", style(status, failed))
                        }),
                )
                .collect::<Vec<_>>()
                .join("\n"),
        );
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

const fn metadata_status(error: Option<&str>) -> &'static str {
    match error {
        None => "ok",
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
