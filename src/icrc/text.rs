//! Module: icrc::text
//!
//! Responsibility: render generic ICRC reports as text.
//! Does not own: live source reads, JSON output, or command parsing.
//! Boundary: formats token metadata and base-unit amounts for humans.

use crate::{
    icrc::model::{
        IcrcAllowanceReport, IcrcArchiveFollowErrorRow, IcrcArchivedBlocksRow,
        IcrcArchivedRangeRow, IcrcArchivesReport, IcrcBalanceReport, IcrcBlockTypesReport,
        IcrcCapabilitiesReport, IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
        IcrcTipCertificateReport, IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenStandardRow,
        IcrcTransactionBlockRow, IcrcTransactionsReport,
    },
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
    token_metadata_text::{
        optional_text, token_metadata_value_text as metadata_value_text, truncate_text_value,
    },
};

const ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;
const ICRC_TIP_CERTIFICATE_HEX_TEXT_LIMIT: usize = 160;
const ICRC_DETAIL_TEXT_LIMIT: usize = 160;
const STANDARD_TABLE_HEADERS: [&str; 2] = ["STANDARD", "URL"];
const LEFT_2_ALIGNMENTS: [ColumnAlign; 2] = [ColumnAlign::Left, ColumnAlign::Left];
const ICRC3_BLOCK_TABLE_HEADERS: [&str; 5] =
    ["INDEX", "TYPE", "KIND", "TIMESTAMP_NS", "AMOUNT_BASE_UNITS"];
const ICRC3_BLOCK_TABLE_ALIGNMENTS: [ColumnAlign; 5] = [
    ColumnAlign::Right,
    ColumnAlign::Left,
    ColumnAlign::Left,
    ColumnAlign::Right,
    ColumnAlign::Right,
];
const ARCHIVE_RANGE_TABLE_ALIGNMENTS: [ColumnAlign; 4] = [
    ColumnAlign::Left,
    ColumnAlign::Left,
    ColumnAlign::Right,
    ColumnAlign::Right,
];

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
    push_table_section(
        &mut lines,
        &report.supported_standards,
        render_standard_rows_table,
    );
    push_table_section(&mut lines, &report.metadata, |rows| {
        render_metadata_rows_table(rows, report.decimals)
    });
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
        format!("follow_archives: {}", report.follow_archives),
        format!("log_length: {}", optional_text(report.log_length.as_ref())),
        format!("returned_blocks: {}", report.blocks.len()),
        format!("archived_callbacks: {}", report.archived_blocks.len()),
        format!(
            "followed_archive_blocks: {}",
            report.followed_archive_blocks.len()
        ),
        format!(
            "archive_follow_errors: {}",
            report.archive_follow_errors.len()
        ),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    push_table_section(&mut lines, &report.blocks, render_transaction_blocks_table);
    push_table_section(
        &mut lines,
        &report.followed_archive_blocks,
        render_followed_archive_blocks_table,
    );
    push_table_section(
        &mut lines,
        &report.archived_blocks,
        render_archive_callbacks_table,
    );
    push_table_section(
        &mut lines,
        &report.archive_follow_errors,
        render_archive_follow_errors_table,
    );
    lines.join("\n")
}

fn render_transaction_blocks_table(blocks: &[IcrcTransactionBlockRow]) -> String {
    render_table(
        &ICRC3_BLOCK_TABLE_HEADERS,
        &blocks
            .iter()
            .map(transaction_block_cells)
            .collect::<Vec<_>>(),
        &ICRC3_BLOCK_TABLE_ALIGNMENTS,
    )
}

fn render_followed_archive_blocks_table(blocks: &[IcrcFollowedArchiveBlockRow]) -> String {
    render_table(
        &[
            "ARCHIVE_CANISTER",
            "METHOD",
            "INDEX",
            "TYPE",
            "KIND",
            "TIMESTAMP_NS",
            "AMOUNT_BASE_UNITS",
        ],
        &blocks
            .iter()
            .map(|block| {
                let [index, block_type, kind, timestamp, amount] =
                    followed_archive_block_cells(block);
                [
                    block.archive_canister_id.clone(),
                    block.callback_method.clone(),
                    index,
                    block_type,
                    kind,
                    timestamp,
                    amount,
                ]
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Right,
        ],
    )
}

fn render_archive_callbacks_table(archives: &[IcrcArchivedBlocksRow]) -> String {
    render_table(
        &["ARCHIVE_CANISTER", "METHOD", "START", "LENGTH"],
        &archives
            .iter()
            .flat_map(|archive| {
                archive.ranges.iter().map(|range| {
                    archive_range_cells(
                        &archive.callback_canister_id,
                        &archive.callback_method,
                        range,
                    )
                })
            })
            .collect::<Vec<_>>(),
        &ARCHIVE_RANGE_TABLE_ALIGNMENTS,
    )
}

fn render_archive_follow_errors_table(errors: &[IcrcArchiveFollowErrorRow]) -> String {
    render_table(
        &["ARCHIVE_CANISTER", "METHOD", "START", "LENGTH", "ERROR"],
        &errors
            .iter()
            .flat_map(|error| {
                error.ranges.iter().map(|range| {
                    let [canister_id, method, start, length] = archive_range_cells(
                        &error.callback_canister_id,
                        &error.callback_method,
                        range,
                    );
                    [
                        canister_id,
                        method,
                        start,
                        length,
                        truncate_text_value(&error.error, ICRC_DETAIL_TEXT_LIMIT),
                    ]
                })
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Right,
            ColumnAlign::Left,
        ],
    )
}

#[must_use]
pub(in crate::icrc) fn icrc_block_types_report_text(report: &IcrcBlockTypesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("block_type_count: {}", report.block_types.len()),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    push_table_section(&mut lines, &report.block_types, |rows| {
        render_table(
            &["BLOCK_TYPE", "URL"],
            &rows
                .iter()
                .map(|block_type| [block_type.block_type.clone(), block_type.url.clone()])
                .collect::<Vec<_>>(),
            &LEFT_2_ALIGNMENTS,
        )
    });
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
    push_table_section(&mut lines, &report.archives, |rows| {
        render_table(
            &["ARCHIVE_CANISTER", "START", "END"],
            &rows
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
        )
    });
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
    push_table_section(
        &mut lines,
        &report.supported_standards,
        render_standard_rows_table,
    );
    push_table_section(
        &mut lines,
        &report.capabilities,
        render_capability_rows_table,
    );
    lines.join("\n")
}

fn push_table_section<T>(lines: &mut Vec<String>, rows: &[T], render: impl FnOnce(&[T]) -> String) {
    if rows.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push(render(rows));
}

fn render_standard_rows_table(standards: &[IcrcTokenStandardRow]) -> String {
    render_table(
        &STANDARD_TABLE_HEADERS,
        &standards
            .iter()
            .map(|standard| [standard.name.clone(), standard.url.clone()])
            .collect::<Vec<_>>(),
        &LEFT_2_ALIGNMENTS,
    )
}

fn render_metadata_rows_table(rows: &[IcrcTokenMetadataRow], decimals: u8) -> String {
    render_table(
        &["METADATA", "TYPE", "VALUE"],
        &rows
            .iter()
            .map(|row| {
                [
                    row.key.clone(),
                    row.value_type.clone(),
                    truncate_text_value(
                        &metadata_value_text(&row.key, &row.value, decimals),
                        ICRC_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
    )
}

fn transaction_block_cells(block: &IcrcTransactionBlockRow) -> [String; 5] {
    block_summary_cells(
        &block.index,
        block.block_type.as_ref(),
        block.transaction_kind.as_ref(),
        block.timestamp_unix_nanos.as_ref(),
        block.amount_base_units.as_ref(),
    )
}

fn followed_archive_block_cells(block: &IcrcFollowedArchiveBlockRow) -> [String; 5] {
    block_summary_cells(
        &block.index,
        block.block_type.as_ref(),
        block.transaction_kind.as_ref(),
        block.timestamp_unix_nanos.as_ref(),
        block.amount_base_units.as_ref(),
    )
}

fn block_summary_cells(
    index: &str,
    block_type: Option<&String>,
    transaction_kind: Option<&String>,
    timestamp_unix_nanos: Option<&String>,
    amount_base_units: Option<&String>,
) -> [String; 5] {
    [
        index.to_string(),
        optional_text(block_type).to_string(),
        optional_text(transaction_kind).to_string(),
        optional_text(timestamp_unix_nanos).to_string(),
        optional_text(amount_base_units).to_string(),
    ]
}

fn archive_range_cells(
    canister_id: &str,
    method: &str,
    range: &IcrcArchivedRangeRow,
) -> [String; 4] {
    [
        canister_id.to_string(),
        method.to_string(),
        range.start.clone(),
        range.length.clone(),
    ]
}

fn render_capability_rows_table(rows: &[IcrcCapabilityRow]) -> String {
    render_table(
        &["CAPABILITY", "METHOD", "STATUS", "DETAIL"],
        &rows
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
    )
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
    optional_truncated_text(detail, ICRC_DETAIL_TEXT_LIMIT)
}
