//! Module: icrc::text::history
//!
//! Responsibility: render ledger-wide ICRC transaction and archive reports as text.
//! Does not own: account history, token metadata, live reads, or JSON output.
//! Boundary: formats bounded ledger blocks and archive evidence for humans.

use super::{LEFT_2_ALIGNMENTS, push_table_section};
use crate::{
    icrc::model::{
        IcrcArchiveFollowErrorRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesReport,
        IcrcBlockTypesReport, IcrcFollowedArchiveBlockRow, IcrcTransactionBlockRow,
        IcrcTransactionsReport,
    },
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, truncate_text},
    token_metadata_text::optional_text,
};

const ICRC_DETAIL_TEXT_LIMIT: usize = 160;
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
pub fn icrc_transactions_report_text(report: &IcrcTransactionsReport) -> String {
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
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
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
                        truncate_text(&error.error, ICRC_DETAIL_TEXT_LIMIT),
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
pub fn icrc_block_types_report_text(report: &IcrcBlockTypesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("block_type_count: {}", report.block_types.len()),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
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
pub fn icrc_archives_report_text(report: &IcrcArchivesReport) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!(
            "from_canister_id: {}",
            optional_text(report.from_canister_id.as_ref())
        ),
        format!("archive_count: {}", report.archives.len()),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
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
        optional_text(block_type),
        optional_text(transaction_kind),
        optional_text(timestamp_unix_nanos),
        optional_text(amount_base_units),
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
