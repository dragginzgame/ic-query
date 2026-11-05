//! Module: icrc::commands::account
//!
//! Responsibility: construct the ICRC account and account-history Clap command tree.
//! Does not own: typed option extraction, command dispatch, report construction, or output.
//! Boundary: keeps live account queries and explicit history collection modes under one family.

use super::options::{
    IcrcAccountOptions, IcrcAllowanceOptions, IcrcTransactionListOptions,
    IcrcTransactionPageOptions, IcrcTransactionRefreshOptions,
};
use crate::cli::common::{
    COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_FORCE_REFRESH, COLLECTION_MODE_LIVE,
    collection_help,
};
use clap::{Args, Command as ClapCommand};

pub(in crate::icrc) fn command() -> ClapCommand {
    ClapCommand::new("account")
        .about("Inspect ICRC account balances, allowances, and transaction history")
        .subcommand(icrc_balance_command())
        .subcommand(icrc_allowance_command())
        .subcommand(icrc_account_transaction_command())
}

pub(in crate::icrc) fn icrc_balance_command() -> ClapCommand {
    let command = ClapCommand::new("balance")

        .about("Show a generic ICRC account balance")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa\n  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ));
    IcrcAccountOptions::augment_args(command)
}

pub(in crate::icrc) fn icrc_allowance_command() -> ClapCommand {
    let command = ClapCommand::new("allowance")

        .about("Show a generic ICRC account allowance")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa\n  icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ));
    IcrcAllowanceOptions::augment_args(command)
}

pub(in crate::icrc) fn icrc_account_transaction_command() -> ClapCommand {
    ClapCommand::new("transaction")
        .about("Inspect live pages and complete cached account transaction history")
        .subcommand(icrc_account_transaction_page_command())
        .subcommand(icrc_account_transaction_list_command())
        .subcommand(icrc_account_transaction_refresh_command())
        .subcommand(icrc_account_transaction_cache_command())
}

pub(in crate::icrc) fn icrc_account_transaction_page_command() -> ClapCommand {
    let command = ClapCommand::new("page")

        .about("Show an ICRC account transaction-history page from its index")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --start 100 --limit 25 --json\n  icq icrc account transaction page ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai",
        ));
    IcrcTransactionPageOptions::augment_args(command)
}

pub(in crate::icrc) fn icrc_account_transaction_list_command() -> ClapCommand {
    let command = ClapCommand::new("list")

        .about("List rows from a complete local ICRC account-history cache")
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            "Examples:\n  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --sort oldest --limit 100 --json",
        ));
    IcrcTransactionListOptions::augment_args(command)
}

pub(in crate::icrc) fn icrc_account_transaction_refresh_command() -> ClapCommand {
    let command = ClapCommand::new("refresh")

        .about("Fetch and atomically cache complete ICRC account history")
        .after_help(collection_help(
            COLLECTION_MODE_FORCE_REFRESH,
            "Examples:\n  icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction refresh ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai --page-size 100 --json",
        ));
    IcrcTransactionRefreshOptions::augment_args(command)
}

pub(in crate::icrc) fn icrc_account_transaction_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .about("Inspect local complete account-history cache state")
        .subcommand(icrc_account_transaction_cache_status_command())
}

pub(in crate::icrc) fn icrc_account_transaction_cache_status_command() -> ClapCommand {
    let command = ClapCommand::new("status")

        .about("Show local account-history cache and latest refresh-attempt status")
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            "Examples:\n  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --json",
        ));
    IcrcAccountOptions::augment_args(command)
}
