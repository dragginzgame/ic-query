use super::{SnsInfoReport, SnsListReport, SnsTokenReport, short_principal};
use crate::{
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
};
use common::{optional_text, token_metadata_value_text, truncate_text_value};

mod common;
mod neurons;
mod params;
mod proposals;
#[cfg(test)]
pub(super) use common::optional_e8s_decimal_text;
pub use neurons::{
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_refresh_report_text, sns_neurons_report_text,
};
pub use params::sns_params_report_text;
pub use proposals::{sns_proposal_report_text, sns_proposals_report_text};

const SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;

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

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!(
            "description: {}",
            optional_text(report.description.as_ref())
        ),
        format!("url: {}", optional_text(report.url.as_ref())),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("swap_canister_id: {}", report.swap_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {error}"));
    }
    lines.join("\n")
}

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

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}
