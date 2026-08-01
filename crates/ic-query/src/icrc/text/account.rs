//! Module: icrc::text::account
//!
//! Responsibility: render ICRC account and cached account-history reports as text.
//! Does not own: ledger-wide reports, live source reads, JSON output, or command parsing.
//! Boundary: formats account identities and base-unit amounts for humans.

use super::push_table_section;
use crate::{
    icrc::model::{
        IcrcAccountRow, IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionListReport,
        IcrcAccountTransactionPageReport, IcrcAccountTransactionRefreshReport,
        IcrcAccountTransactionRow, IcrcAllowanceReport, IcrcBalanceReport,
    },
    table::{ColumnAlign, render_table},
    text_value::sanitize_text,
    token_amount::base_units_decimal_text,
    token_metadata_text::optional_text,
};

const ICRC_ACCOUNT_TRANSACTION_TABLE_ALIGNMENTS: [ColumnAlign; 8] = [
    ColumnAlign::Right,
    ColumnAlign::Left,
    ColumnAlign::Right,
    ColumnAlign::Right,
    ColumnAlign::Right,
    ColumnAlign::Left,
    ColumnAlign::Left,
    ColumnAlign::Left,
];
#[must_use]
pub fn icrc_balance_report_text(report: &IcrcBalanceReport) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!("token_symbol: {}", sanitize_text(&report.token_symbol)),
        format!("decimals: {}", report.decimals),
        format!(
            "balance: {} {}",
            base_units_decimal_text(&report.balance, report.decimals),
            sanitize_text(&report.token_symbol)
        ),
        format!("balance_base_units: {}", report.balance),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ]
    .join("\n")
}

#[must_use]
pub fn icrc_allowance_report_text(report: &IcrcAllowanceReport) -> String {
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
        format!("token_symbol: {}", sanitize_text(&report.token_symbol)),
        format!("decimals: {}", report.decimals),
        format!(
            "allowance: {} {}",
            base_units_decimal_text(&report.allowance, report.decimals),
            sanitize_text(&report.token_symbol)
        ),
        format!("allowance_base_units: {}", report.allowance),
        format!(
            "expires_at_unix_nanos: {}",
            optional_text(report.expires_at_unix_nanos.as_ref())
        ),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ]
    .join("\n")
}

/// Renders an ICRC index account-transaction report as human-facing text.
#[must_use]
pub fn icrc_account_transaction_page_report_text(
    report: &IcrcAccountTransactionPageReport,
) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!(
            "requested_start: {}",
            optional_text(report.requested_start.as_ref())
        ),
        format!("requested_limit: {}", report.requested_limit),
        format!("next_start: {}", optional_text(report.next_start.as_ref())),
        format!(
            "oldest_transaction_id: {}",
            optional_text(report.oldest_transaction_id.as_ref())
        ),
        format!(
            "balance: {} {}",
            base_units_decimal_text(&report.balance, report.decimals),
            sanitize_text(&report.token_symbol)
        ),
        format!("balance_base_units: {}", report.balance),
        format!("returned_transactions: {}", report.transactions.len()),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ];
    push_table_section(&mut lines, &report.transactions, |transactions| {
        render_account_transactions_table(
            transactions,
            report.decimals,
            &sanitize_text(&report.token_symbol),
        )
    });
    lines.join("\n")
}

/// Renders a cache-only complete account-history list view.
#[must_use]
pub fn icrc_account_transaction_list_report_text(
    report: &IcrcAccountTransactionListReport,
) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!("requested_limit: {}", report.requested_limit),
        format!("sort: {}", report.sort),
        format!(
            "returned_transactions: {}",
            report.returned_transaction_count
        ),
        format!("total_transactions: {}", report.total_transaction_count),
        format!(
            "newest_transaction_id: {}",
            optional_text(report.newest_transaction_id.as_ref())
        ),
        format!(
            "oldest_transaction_id: {}",
            optional_text(report.oldest_transaction_id.as_ref())
        ),
        format!(
            "balance: {} {}",
            base_units_decimal_text(&report.balance, report.decimals),
            sanitize_text(&report.token_symbol)
        ),
        format!("balance_base_units: {}", report.balance),
        format!("complete: {}", report.complete),
        format!(
            "point_in_time_guaranteed: {}",
            report.point_in_time_guaranteed
        ),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!(
            "collection_started_at: {}",
            sanitize_text(&report.collection_started_at)
        ),
        format!(
            "collection_completed_at: {}",
            sanitize_text(&report.collection_completed_at)
        ),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
    ];
    push_table_section(&mut lines, &report.transactions, |transactions| {
        render_account_transactions_table(
            transactions,
            report.decimals,
            &sanitize_text(&report.token_symbol),
        )
    });
    lines.join("\n")
}

/// Renders a complete account-history refresh outcome.
#[must_use]
pub fn icrc_account_transaction_refresh_report_text(
    report: &IcrcAccountTransactionRefreshReport,
) -> String {
    [
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!("transaction_count: {}", report.transaction_count),
        format!(
            "newest_transaction_id: {}",
            optional_text(report.newest_transaction_id.as_ref())
        ),
        format!(
            "oldest_transaction_id: {}",
            optional_text(report.oldest_transaction_id.as_ref())
        ),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!(
            "point_in_time_guaranteed: {}",
            report.point_in_time_guaranteed
        ),
        format!(
            "replaced_existing_cache: {}",
            report.replaced_existing_cache
        ),
        format!(
            "attempt_finalization_error: {}",
            optional_text(report.attempt_finalization_error.as_ref())
        ),
        format!(
            "collection_started_at: {}",
            sanitize_text(&report.collection_started_at)
        ),
        format!(
            "collection_completed_at: {}",
            sanitize_text(&report.collection_completed_at)
        ),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

/// Renders local complete account-history cache status.
#[must_use]
pub fn icrc_account_transaction_cache_status_report_text(
    report: &IcrcAccountTransactionCacheStatusReport,
) -> String {
    let mut lines = vec![
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("account_owner: {}", report.account_owner),
        format!(
            "subaccount_hex: {}",
            optional_text(report.subaccount_hex.as_ref())
        ),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("found: {}", report.found),
        format!(
            "expected_cache_path: {}",
            sanitize_text(&report.expected_cache_path)
        ),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("cache_status: {}", cache.cache_status),
            format!("cache_error: {}", optional_text(cache.cache_error.as_ref())),
            format!(
                "index_canister_id: {}",
                optional_text(cache.index_canister_id.as_ref())
            ),
            format!("transaction_count: {}", cache.transaction_count),
            format!(
                "newest_transaction_id: {}",
                optional_text(cache.newest_transaction_id.as_ref())
            ),
            format!(
                "oldest_transaction_id: {}",
                optional_text(cache.oldest_transaction_id.as_ref())
            ),
            format!("page_size: {}", cache.page_size),
            format!("page_count: {}", cache.page_count),
            format!("complete: {}", cache.complete),
            format!(
                "point_in_time_guaranteed: {}",
                cache.point_in_time_guaranteed
            ),
            format!(
                "collection_started_at: {}",
                sanitize_text(&cache.collection_started_at)
            ),
            format!(
                "collection_completed_at: {}",
                sanitize_text(&cache.collection_completed_at)
            ),
        ]);
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.extend([
            format!("latest_attempt_status: {}", attempt.status),
            format!("latest_attempt_started_at: {}", attempt.started_at),
            format!("latest_attempt_updated_at: {}", attempt.updated_at),
            format!(
                "latest_attempt_index_canister_id: {}",
                optional_text(attempt.index_canister_id.as_ref())
            ),
            format!("latest_attempt_page_size: {}", attempt.page_size),
            format!("latest_attempt_pages: {}", attempt.pages_fetched),
            format!("latest_attempt_rows: {}", attempt.rows_fetched),
            format!(
                "latest_attempt_cursor: {}",
                optional_text(attempt.last_cursor.as_ref())
            ),
            format!(
                "latest_attempt_error: {}",
                optional_text(attempt.last_error.as_ref())
            ),
        ]);
    }
    lines.join("\n")
}

fn render_account_transactions_table(
    transactions: &[IcrcAccountTransactionRow],
    decimals: u8,
    token_symbol: &str,
) -> String {
    render_table(
        &[
            "ID",
            "KIND",
            "TIMESTAMP_NS",
            "AMOUNT",
            "FEE",
            "FROM",
            "TO",
            "SPENDER",
        ],
        &transactions
            .iter()
            .map(|transaction| {
                [
                    transaction.id.clone(),
                    sanitize_text(&transaction.kind),
                    optional_text(transaction.timestamp_unix_nanos.as_ref()),
                    optional_amount_text(
                        transaction.amount_base_units.as_ref(),
                        decimals,
                        token_symbol,
                    ),
                    optional_amount_text(
                        transaction.fee_base_units.as_ref(),
                        decimals,
                        token_symbol,
                    ),
                    optional_account_text(transaction.from.as_ref()),
                    optional_account_text(transaction.to.as_ref()),
                    optional_account_text(transaction.spender.as_ref()),
                ]
            })
            .collect::<Vec<_>>(),
        &ICRC_ACCOUNT_TRANSACTION_TABLE_ALIGNMENTS,
    )
}

fn optional_amount_text(amount: Option<&String>, decimals: u8, token_symbol: &str) -> String {
    amount.map_or_else(
        || "-".to_string(),
        |amount| {
            format!(
                "{} {token_symbol}",
                base_units_decimal_text(amount, decimals)
            )
        },
    )
}

fn optional_account_text(account: Option<&IcrcAccountRow>) -> String {
    account.map_or_else(|| "-".to_string(), account_text)
}

fn account_text(account: &IcrcAccountRow) -> String {
    if let Some(account_identifier) = account.account_identifier.as_ref() {
        return sanitize_text(account_identifier);
    }
    let Some(owner) = account.owner.as_ref() else {
        return "-".to_string();
    };
    account.subaccount_hex.as_ref().map_or_else(
        || sanitize_text(owner),
        |subaccount| format!("{}:{}", sanitize_text(owner), sanitize_text(subaccount)),
    )
}
