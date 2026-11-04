//! Module: sns::list
//!
//! Responsibility: render deployed SNS list reports as text.
//! Does not own: report construction, list sorting, lookup ids, or JSON output.
//! Boundary: formats list DTO rows into compact or verbose human-readable tables.

use crate::sns::labels::sns_swap_lifecycle_name;
use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::table::Alignment;

use crate::sns::short_principal;
use ic_query::sns::SnsListReport;

use crate::text_value::yes_no;

#[must_use]
/// Render the SNS list report as human-facing text.
pub fn sns_list_report_text(report: &SnsListReport, options: crate::TextOptions) -> String {
    sns_list_report_text_with_metadata_style(report, options, |status, _| status.to_string())
}

/// Render an SNS list with caller-selected styling for its metadata status cells.
///
/// The callback receives the status label and whether metadata collection failed.
/// It may add trusted styling, such as ANSI colours, after escaping and alignment.
/// The caller owns terminal policy; this renderer never selects an output sink.
#[must_use]
pub fn sns_list_report_text_with_metadata_style(
    report: &SnsListReport,
    options: crate::TextOptions,
    mut style: impl FnMut(&str, bool) -> String,
) -> String {
    let mut lines = vec![
        field("network", &report.network),
        field("sns_wasm_canister_id", &(report.sns_wasm_canister_id)),
        field("all_lifecycles", &(yes_no(report.all_lifecycles))),
        field("catalog_sns_count", &(report.catalog_sns_count)),
        field("excluded_sns_count", &(report.excluded_sns_count)),
        field("sns_count", &(report.sns_instances.len())),
        field("fetched_at", &report.fetched_at),
    ];
    crate::sns::common::push_report_provenance_lines(&mut lines, &report.provenance);
    lines.push(field("source_endpoint", &report.source_endpoint));
    lines.push(field("sort", &(report.sort)));
    lines.push(field("metadata_errors", &(report.metadata_error_count)));
    lines.push(field("lifecycle_errors", &(report.lifecycle_error_count)));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        let table = table(
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
                        principal_for_list(&sns.canisters.root_canister_id, options.verbose),
                        principal_for_list(&sns.canisters.governance_canister_id, options.verbose),
                        principal_for_list(&sns.canisters.ledger_canister_id, options.verbose),
                        principal_for_list(&sns.canisters.swap_canister_id, options.verbose),
                        principal_for_list(&sns.canisters.index_canister_id, options.verbose),
                        lifecycle_status(
                            sns_swap_lifecycle_name(sns.lifecycle.value()),
                            sns.lifecycle.error(),
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
