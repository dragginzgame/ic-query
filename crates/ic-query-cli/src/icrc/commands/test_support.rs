//! Module: icrc::commands::test_support
//!
//! Responsibility: expose typed ICRC command parsers and usage to unit tests.
//! Does not own: production parsing, dispatch, or behavior assertions.
//! Boundary: keeps test-only access out of the production command surface.

use super::{
    IcrcAccountTransactionCacheOptions, IcrcAccountTransactionListOptions,
    IcrcAccountTransactionPageOptions, IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions,
    IcrcArchivesOptions, IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
    account::{
        command as icrc_account_command, icrc_account_transaction_cache_command,
        icrc_account_transaction_cache_status_command, icrc_account_transaction_command,
        icrc_account_transaction_list_command, icrc_account_transaction_page_command,
        icrc_account_transaction_refresh_command, icrc_allowance_command, icrc_balance_command,
    },
    command,
    ledger::{
        command as icrc_ledger_command, icrc_archives_command, icrc_block_types_command,
        icrc_capabilities_command, icrc_index_command, icrc_tip_certificate_command,
        icrc_token_command, icrc_transactions_command,
    },
};
use crate::cli::clap::render_help;

pub(in crate::icrc) fn parse_token_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_token_command, "parse ICRC token options")
}

pub(in crate::icrc) fn parse_capabilities_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(
        args,
        icrc_capabilities_command,
        "parse ICRC capabilities options",
    )
}

pub(in crate::icrc) fn parse_balance_options(args: &[&str]) -> IcrcBalanceOptions {
    IcrcBalanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC balance options")
}

pub(in crate::icrc) fn parse_allowance_options(args: &[&str]) -> IcrcAllowanceOptions {
    IcrcAllowanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC allowance options")
}

pub(in crate::icrc) fn parse_account_transaction_page_options(
    args: &[&str],
) -> IcrcAccountTransactionPageOptions {
    IcrcAccountTransactionPageOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC account transaction page options")
}

pub(in crate::icrc) fn parse_account_transaction_list_options(
    args: &[&str],
) -> IcrcAccountTransactionListOptions {
    IcrcAccountTransactionListOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC account transaction list options")
}

pub(in crate::icrc) fn parse_account_transaction_refresh_options(
    args: &[&str],
) -> IcrcAccountTransactionRefreshOptions {
    IcrcAccountTransactionRefreshOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC account transaction refresh options")
}

pub(in crate::icrc) fn parse_account_transaction_cache_options(
    args: &[&str],
) -> IcrcAccountTransactionCacheOptions {
    IcrcAccountTransactionCacheOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC account transaction cache options")
}

pub(in crate::icrc) fn parse_index_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_index_command, "parse ICRC index options")
}

pub(in crate::icrc) fn parse_transactions_options(args: &[&str]) -> IcrcTransactionsOptions {
    IcrcTransactionsOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC transactions options")
}

pub(in crate::icrc) fn parse_block_types_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(
        args,
        icrc_block_types_command,
        "parse ICRC block types options",
    )
}

pub(in crate::icrc) fn parse_archives_options(args: &[&str]) -> IcrcArchivesOptions {
    IcrcArchivesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
        .expect("parse ICRC archives options")
}

pub(in crate::icrc) fn parse_tip_certificate_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(
        args,
        icrc_tip_certificate_command,
        "parse ICRC tip certificate options",
    )
}

fn parse_ledger_options(
    args: &[&str],
    command: fn() -> clap::Command,
    expectation: &str,
) -> IcrcLedgerOptions {
    IcrcLedgerOptions::parse(args.iter().copied().map(std::ffi::OsString::from), command)
        .expect(expectation)
}

pub(in crate::icrc) fn root_usage() -> String {
    render_help(command())
}

pub(in crate::icrc) fn ledger_usage() -> String {
    render_help(icrc_ledger_command())
}

pub(in crate::icrc) fn account_usage() -> String {
    render_help(icrc_account_command())
}

pub(in crate::icrc) fn token_usage() -> String {
    render_help(icrc_token_command())
}

pub(in crate::icrc) fn capabilities_usage() -> String {
    render_help(icrc_capabilities_command())
}

pub(in crate::icrc) fn balance_usage() -> String {
    render_help(icrc_balance_command())
}

pub(in crate::icrc) fn allowance_usage() -> String {
    render_help(icrc_allowance_command())
}

pub(in crate::icrc) fn account_transaction_usage() -> String {
    render_help(icrc_account_transaction_command())
}

pub(in crate::icrc) fn account_transaction_page_usage() -> String {
    render_help(icrc_account_transaction_page_command())
}

pub(in crate::icrc) fn account_transaction_list_usage() -> String {
    render_help(icrc_account_transaction_list_command())
}

pub(in crate::icrc) fn account_transaction_refresh_usage() -> String {
    render_help(icrc_account_transaction_refresh_command())
}

pub(in crate::icrc) fn account_transaction_cache_usage() -> String {
    render_help(icrc_account_transaction_cache_command())
}

pub(in crate::icrc) fn account_transaction_cache_status_usage() -> String {
    render_help(icrc_account_transaction_cache_status_command())
}

pub(in crate::icrc) fn index_usage() -> String {
    render_help(icrc_index_command())
}

pub(in crate::icrc) fn transactions_usage() -> String {
    render_help(icrc_transactions_command())
}

pub(in crate::icrc) fn block_types_usage() -> String {
    render_help(icrc_block_types_command())
}

pub(in crate::icrc) fn archives_usage() -> String {
    render_help(icrc_archives_command())
}

pub(in crate::icrc) fn tip_certificate_usage() -> String {
    render_help(icrc_tip_certificate_command())
}
