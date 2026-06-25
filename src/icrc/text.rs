//! Module: icrc::text
//!
//! Responsibility: render generic ICRC reports as text.
//! Does not own: live source reads, JSON output, or command parsing.
//! Boundary: formats token metadata and base-unit amounts for humans.

use crate::{
    icrc::model::{
        IcrcAllowanceReport, IcrcArchivesReport, IcrcBalanceReport, IcrcBlockTypesReport,
        IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcIndexReport, IcrcTipCertificateReport,
        IcrcTokenReport, IcrcTransactionsReport,
    },
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
    token_metadata_text::{
        optional_text, token_metadata_value_text as metadata_value_text, truncate_text_value,
    },
};

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;
const ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT: usize = 160;
const ICRC_CAPABILITY_DETAIL_TEXT_LIMIT: usize = 160;

#[must_use]
pub(in crate::icrc) fn icrc_token_report_text(report: &IcrcTokenReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
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
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
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
                            &metadata_value_text(&row.key, &row.value, report.decimals),
                            ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_balance_report_text(report: &IcrcBalanceReport) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!("token_symbol: {}", report.token_symbol),
        format!("decimals: {}", report.decimals),
        format!(
            "balance: {} {}",
            base_units_decimal_text(&report.balance, report.decimals),
            report.token_symbol
        ),
        format!("balance_base_units: {}", report.balance),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_allowance_report_text(report: &IcrcAllowanceReport) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "account_subaccount_hex: {}",
            optional_text(report.account_subaccount_hex.as_ref())
        ),
        format!("spender_owner: {}", report.spender_owner),
        format!(
            "spender_subaccount_hex: {}",
            optional_text(report.spender_subaccount_hex.as_ref())
        ),
        format!("token_symbol: {}", report.token_symbol),
        format!("decimals: {}", report.decimals),
        format!(
            "allowance: {} {}",
            base_units_decimal_text(&report.allowance, report.decimals),
            report.token_symbol
        ),
        format!("allowance_base_units: {}", report.allowance),
        format!(
            "expires_at_unix_nanos: {}",
            optional_text(report.expires_at_unix_nanos.as_ref())
        ),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_index_report_text(report: &IcrcIndexReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!(
            "index_canister_id: {}",
            optional_text(report.index_canister_id.as_ref())
        ),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.index_error.as_deref() {
        lines.push(format!("index_error: {error}"));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_transactions_report_text(report: &IcrcTransactionsReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("requested_start: {}", report.requested_start),
        format!("requested_limit: {}", report.requested_limit),
        format!("log_length: {}", optional_text(report.log_length.as_ref())),
        format!("returned_blocks: {}", report.blocks.len()),
        format!("archived_callbacks: {}", report.archived_blocks.len()),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.blocks.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["INDEX", "TYPE", "KIND", "TIMESTAMP_NS", "AMOUNT_BASE_UNITS"],
            &report
                .blocks
                .iter()
                .map(|block| {
                    [
                        block.index.clone(),
                        optional_text(block.block_type.as_ref()).to_string(),
                        optional_text(block.transaction_kind.as_ref()).to_string(),
                        optional_text(block.timestamp_unix_nanos.as_ref()).to_string(),
                        optional_text(block.amount_base_units.as_ref()).to_string(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ));
    }
    if !report.archived_blocks.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ARCHIVE_CANISTER", "METHOD", "START", "LENGTH"],
            &report
                .archived_blocks
                .iter()
                .flat_map(|archive| {
                    archive.ranges.iter().map(|range| {
                        [
                            archive.callback_canister_id.clone(),
                            archive.callback_method.clone(),
                            range.start.clone(),
                            range.length.clone(),
                        ]
                    })
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_block_types_report_text(report: &IcrcBlockTypesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("block_type_count: {}", report.block_types.len()),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.block_types.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["BLOCK_TYPE", "URL"],
            &report
                .block_types
                .iter()
                .map(|block_type| [block_type.block_type.clone(), block_type.url.clone()])
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_archives_report_text(report: &IcrcArchivesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!(
            "from_canister_id: {}",
            optional_text(report.from_canister_id.as_ref())
        ),
        format!("archive_count: {}", report.archives.len()),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.archives.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ARCHIVE_CANISTER", "START", "END"],
            &report
                .archives
                .iter()
                .map(|archive| {
                    [
                        archive.canister_id.clone(),
                        archive.start.clone(),
                        archive.end.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Right, ColumnAlign::Right],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_tip_certificate_report_text(
    report: &IcrcTipCertificateReport,
) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("certificate_present: {}", report.certificate_present),
        format!(
            "certificate_bytes: {}",
            optional_usize_text(report.certificate_bytes)
        ),
        format!(
            "hash_tree_bytes: {}",
            optional_usize_text(report.hash_tree_bytes)
        ),
        format!(
            "certificate_hex: {}",
            optional_truncated_text(
                report.certificate_hex.as_ref(),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
        format!(
            "hash_tree_hex: {}",
            optional_truncated_text(
                report.hash_tree_hex.as_ref(),
                ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT
            )
        ),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

#[must_use]
pub(in crate::icrc) fn icrc_capabilities_report_text(report: &IcrcCapabilitiesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("standard_count: {}", report.supported_standards.len()),
        format!("capability_count: {}", report.capabilities.len()),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
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
    if !report.capabilities.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["CAPABILITY", "METHOD", "STATUS", "DETAIL"],
            &report
                .capabilities
                .iter()
                .map(|capability| {
                    [
                        capability.capability.clone(),
                        capability.method.clone(),
                        capability.status.clone(),
                        capability_detail_text(capability),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

fn optional_usize_text(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_truncated_text(value: Option<&String>, limit: usize) -> String {
    value.map_or_else(
        || "-".to_string(),
        |value| truncate_text_value(value, limit),
    )
}

fn capability_detail_text(row: &IcrcCapabilityRow) -> String {
    let detail = row.details.as_ref().or(row.error.as_ref());
    optional_truncated_text(detail, ICRC_CAPABILITY_DETAIL_TEXT_LIMIT)
}
