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
use crate::cli::clap::{parse_matches_or_usage, render_help};
use clap::{ArgMatches, Command as ClapCommand};
use std::ffi::OsString;

pub(in crate::icrc) fn parse_token_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_token_command)
}

pub(in crate::icrc) fn parse_capabilities_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_capabilities_command)
}

pub(in crate::icrc) fn parse_balance_options(args: &[&str]) -> IcrcBalanceOptions {
    parse_options(args, icrc_balance_command, IcrcBalanceOptions::from_matches)
}

pub(in crate::icrc) fn parse_allowance_options(args: &[&str]) -> IcrcAllowanceOptions {
    parse_options(
        args,
        icrc_allowance_command,
        IcrcAllowanceOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_account_transaction_page_options(
    args: &[&str],
) -> IcrcAccountTransactionPageOptions {
    parse_options(
        args,
        icrc_account_transaction_page_command,
        IcrcAccountTransactionPageOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_account_transaction_list_options(
    args: &[&str],
) -> IcrcAccountTransactionListOptions {
    parse_options(
        args,
        icrc_account_transaction_list_command,
        IcrcAccountTransactionListOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_account_transaction_refresh_options(
    args: &[&str],
) -> IcrcAccountTransactionRefreshOptions {
    parse_options(
        args,
        icrc_account_transaction_refresh_command,
        IcrcAccountTransactionRefreshOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_account_transaction_cache_options(
    args: &[&str],
) -> IcrcAccountTransactionCacheOptions {
    parse_options(
        args,
        icrc_account_transaction_cache_status_command,
        IcrcAccountTransactionCacheOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_index_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_index_command)
}

pub(in crate::icrc) fn parse_transactions_options(args: &[&str]) -> IcrcTransactionsOptions {
    parse_options(
        args,
        icrc_transactions_command,
        IcrcTransactionsOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_block_types_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_block_types_command)
}

pub(in crate::icrc) fn parse_archives_options(args: &[&str]) -> IcrcArchivesOptions {
    parse_options(
        args,
        icrc_archives_command,
        IcrcArchivesOptions::from_matches,
    )
}

pub(in crate::icrc) fn parse_tip_certificate_options(args: &[&str]) -> IcrcLedgerOptions {
    parse_ledger_options(args, icrc_tip_certificate_command)
}

fn parse_ledger_options(args: &[&str], command: fn() -> ClapCommand) -> IcrcLedgerOptions {
    parse_options(args, command, IcrcLedgerOptions::from_matches)
}

fn parse_options<Options>(
    args: &[&str],
    command: fn() -> ClapCommand,
    from_matches: fn(&ArgMatches) -> Options,
) -> Options {
    let matches = parse_matches_or_usage(command(), args.iter().copied().map(OsString::from))
        .expect("parse ICRC test options");
    from_matches(&matches)
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
