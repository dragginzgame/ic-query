//! Module: icrc::account
//!
//! Responsibility: render ICRC account and cached account-history reports as text.
//! Does not own: ledger-wide reports, live source reads, JSON output, or command parsing.
//! Boundary: formats account identities and base-unit amounts for humans.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::icrc::push_table_section;
use crate::text_value::optional_text;
use crate::token_amount::base_units_decimal_text;
use ic_query::icrc::IcrcAccountRow;
use ic_query::icrc::IcrcAccountTransactionCacheStatusReport;
use ic_query::icrc::IcrcAccountTransactionListReport;
use ic_query::icrc::IcrcAccountTransactionPageReport;
use ic_query::icrc::IcrcAccountTransactionRefreshReport;
use ic_query::icrc::IcrcAccountTransactionRow;
use ic_query::icrc::IcrcAllowanceReport;
use ic_query::icrc::IcrcBalanceReport;

const ICRC_ACCOUNT_TRANSACTION_TABLE_ALIGNMENTS: [Alignment; 8] = [
    Alignment::Right,
    Alignment::Left,
    Alignment::Right,
    Alignment::Right,
    Alignment::Right,
    Alignment::Left,
    Alignment::Left,
    Alignment::Left,
];
#[must_use]
/// Render the ICRC balance report as human-facing text.
pub fn icrc_balance_report_text(report: &IcrcBalanceReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        ledger_canister_id = report.provenance.ledger_canister_id,
        account_owner,
        subaccount_hex = optional_text(report.subaccount_hex.as_ref()),
        token_symbol = report.value.token_symbol,
        decimals = report.value.decimals,
    ));
    lines.push(format!(
        "balance: {} {}",
        base_units_decimal_text(&report.value.balance, report.value.decimals),
        terminal_text(&report.value.token_symbol)
    ));
    lines.extend(crate::text_value::report_fields!(report;
        balance_base_units = report.value.balance,
    ));
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    lines.join("\n")
}

#[must_use]
/// Render the ICRC allowance report as human-facing text.
pub fn icrc_allowance_report_text(report: &IcrcAllowanceReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        ledger_canister_id = report.provenance.ledger_canister_id,
        account_owner,
        account_subaccount_hex = optional_text(report.account_subaccount_hex.as_ref()),
        spender_owner,
        spender_subaccount_hex = optional_text(report.spender_subaccount_hex.as_ref()),
        token_symbol = report.value.token_symbol,
        decimals = report.value.decimals,
    ));
    lines.push(format!(
        "allowance: {} {}",
        base_units_decimal_text(&report.value.allowance, report.value.decimals),
        terminal_text(&report.value.token_symbol)
    ));
    lines.extend(crate::text_value::report_fields!(report;
        allowance_base_units = report.value.allowance,
        expires_at_unix_nanos = optional_text(report.value.expires_at_unix_nanos.as_ref()),
    ));
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    lines.join("\n")
}

/// Renders an ICRC index account-transaction report as human-facing text.
#[must_use]
pub fn icrc_account_transaction_page_report_text(
    report: &IcrcAccountTransactionPageReport,
) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        ledger_canister_id = report.provenance.ledger_canister_id,
        index_canister_id = report.page.index_canister_id,
        account_owner,
        subaccount_hex = optional_text(report.subaccount_hex.as_ref()),
        requested_start = optional_text(report.requested_start.as_ref()),
        requested_limit,
        next_start = optional_text(report.page.next_start.as_ref()),
        oldest_transaction_id = optional_text(report.page.oldest_transaction_id.as_ref()),
    ));
    lines.push(format!(
        "balance: {} {}",
        base_units_decimal_text(&report.page.balance, report.page.decimals),
        terminal_text(&report.page.token_symbol)
    ));
    lines.extend(crate::text_value::report_fields!(report;
        balance_base_units = report.page.balance,
        returned_transactions = report.page.transactions.len(),
    ));
    lines.extend(crate::icrc::acquisition_lines(&report.provenance));
    push_table_section(&mut lines, &report.page.transactions, |transactions| {
        render_account_transactions_table(
            transactions,
            report.page.decimals,
            &terminal_text(&report.page.token_symbol),
        )
    });
    lines.join("\n")
}

/// Renders a cache-only complete account-history list view.
#[must_use]
pub fn icrc_account_transaction_list_report_text(
    report: &IcrcAccountTransactionListReport,
) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        ledger_canister_id = report.context.ledger_canister_id,
        index_canister_id = report.context.index_canister_id,
        account_owner = report.context.account_owner,
        subaccount_hex = optional_text(report.context.subaccount_hex.as_ref()),
        requested_limit,
        sort,
        returned_transactions = report.transactions.len(),
        total_transactions = report.total_transaction_count,
        newest_transaction_id = optional_text(report.newest_transaction_id.as_ref()),
        oldest_transaction_id = optional_text(report.oldest_transaction_id.as_ref()),
    ));
    lines.push(format!(
        "balance: {} {}",
        base_units_decimal_text(&report.balance, report.decimals),
        terminal_text(&report.token_symbol)
    ));
    lines.extend(crate::text_value::report_fields!(report;
        balance_base_units = report.balance,
        complete,
        point_in_time_guaranteed,
        page_size,
        page_count,
        collection_started_at = report.context.collection_started_at,
        collection_completed_at = report.context.collection_completed_at,
        source_endpoint = report.context.source_endpoint,
        cache_path,
    ));
    push_table_section(&mut lines, &report.transactions, |transactions| {
        render_account_transactions_table(
            transactions,
            report.decimals,
            &terminal_text(&report.token_symbol),
        )
    });
    lines.join("\n")
}

/// Renders a complete account-history refresh outcome.
#[must_use]
pub fn icrc_account_transaction_refresh_report_text(
    report: &IcrcAccountTransactionRefreshReport,
) -> String {
    crate::text_value::report_fields!(report;
        ledger_canister_id = report.context.ledger_canister_id,
        index_canister_id = report.context.index_canister_id,
        account_owner = report.context.account_owner,
        subaccount_hex = optional_text(report.context.subaccount_hex.as_ref()),
        transaction_count,
        newest_transaction_id = optional_text(report.newest_transaction_id.as_ref()),
        oldest_transaction_id = optional_text(report.oldest_transaction_id.as_ref()),
        page_size,
        page_count,
        point_in_time_guaranteed,
        replaced_existing_cache,
        attempt_finalization_error = optional_text(report.attempt_finalization_error.as_ref()),
        collection_started_at = report.context.collection_started_at,
        collection_completed_at = report.context.collection_completed_at,
        source_endpoint = report.context.source_endpoint,
        cache_path,
        refresh_attempt_path,
        refresh_lock_path,
    )
    .join("\n")
}

/// Renders local complete account-history cache status.
#[must_use]
pub fn icrc_account_transaction_cache_status_report_text(
    report: &IcrcAccountTransactionCacheStatusReport,
) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        ledger_canister_id,
        account_owner,
        subaccount_hex = optional_text(report.subaccount_hex.as_ref()),
        source_endpoint,
        found,
        expected_cache_path,
        refresh_attempt_path,
        refresh_lock_path,
    ));
    match report.cache.as_ref() {
        Some(ic_query::icrc::IcrcAccountTransactionCacheSummary::Valid { data: cache }) => {
            lines.extend(crate::text_value::report_fields!(cache;
                cache_status = "valid",
                cache_error = "-",
                index_canister_id,
                transaction_count,
                newest_transaction_id = optional_text(cache.newest_transaction_id.as_ref()),
                oldest_transaction_id = optional_text(cache.oldest_transaction_id.as_ref()),
                page_size,
                page_count,
                complete = true,
                point_in_time_guaranteed,
                collection_started_at,
                collection_completed_at,
            ));
        }
        Some(ic_query::icrc::IcrcAccountTransactionCacheSummary::Invalid {
            cache_path,
            cache_error,
        }) => {
            lines.extend([
                "cache_status: invalid".to_string(),
                field("cache_path", &(cache_path)),
                field("cache_error", &(cache_error)),
            ]);
        }
        None => {}
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(field(
            "attempt_index_canister_id",
            &optional_text(attempt.index_canister_id.as_ref()),
        ));
        lines.extend(crate::report::refresh_attempt_lines(&attempt.lifecycle));
    }
    lines.join("\n")
}

fn render_account_transactions_table(
    transactions: &[IcrcAccountTransactionRow],
    decimals: u8,
    token_symbol: &str,
) -> String {
    table(
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
                    terminal_text(&transaction.kind).into_owned(),
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
    match account {
        IcrcAccountRow::Legacy { account_identifier } => {
            terminal_text(account_identifier).into_owned()
        }
        IcrcAccountRow::Structured {
            owner,
            subaccount_hex,
        } => subaccount_hex.as_ref().map_or_else(
            || terminal_text(owner).into_owned(),
            |subaccount| format!("{}:{}", terminal_text(owner), terminal_text(subaccount)),
        ),
    }
}
