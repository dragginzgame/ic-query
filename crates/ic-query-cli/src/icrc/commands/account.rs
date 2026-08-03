//! Module: icrc::commands::account
//!
//! Responsibility: construct the ICRC account and account-history Clap command tree.
//! Does not own: typed option extraction, command dispatch, report construction, or output.
//! Boundary: keeps live account queries and explicit history collection modes under one family.

use super::{
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_PAGE_SIZE, DEFAULT_ICRC_TRANSACTIONS_LIMIT,
    INDEX_CANISTER_ID_ARG, LIMIT_ARG, MAX_ICRC_TRANSACTIONS_LIMIT, MAX_PAGES_ARG,
    OWNER_PRINCIPAL_ARG, OWNER_SUBACCOUNT_ARG, PAGE_SIZE_ARG, PRINCIPAL_ARG, SORT_ARG,
    SPENDER_PRINCIPAL_ARG, SPENDER_SUBACCOUNT_ARG, START_ARG, SUBACCOUNT_ARG,
    ledger_canister_id_arg, principal_arg, subaccount_arg, with_common_icrc_options,
};
use crate::cli::{
    clap::value_arg,
    common::{
        COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_FORCE_REFRESH, COLLECTION_MODE_LIVE,
        collection_help,
    },
};
use clap::{
    Command as ClapCommand,
    builder::{RangedU64ValueParser, ValueParser},
};
use ic_query::icrc::ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE;

pub(in crate::icrc) fn command() -> ClapCommand {
    ClapCommand::new("account")
        .bin_name("icq icrc account")
        .about("Inspect ICRC account balances, allowances, and transaction history")
        .subcommand(icrc_balance_command())
        .subcommand(icrc_allowance_command())
        .subcommand(icrc_account_transaction_command())
}

pub(in crate::icrc) fn icrc_balance_command() -> ClapCommand {
    let command = ClapCommand::new("balance")
        .bin_name("icq icrc account balance")
        .about("Show a generic ICRC account balance")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa\n  icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .arg(ledger_canister_id_arg())
        .arg(principal_arg(PRINCIPAL_ARG, "Account owner principal"))
        .arg(subaccount_arg(
            SUBACCOUNT_ARG,
            "Optional 32-byte ICRC subaccount as hex",
        ));
    with_common_icrc_options(command)
}

pub(in crate::icrc) fn icrc_allowance_command() -> ClapCommand {
    let command = ClapCommand::new("allowance")
        .bin_name("icq icrc account allowance")
        .about("Show a generic ICRC account allowance")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa\n  icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .arg(ledger_canister_id_arg())
        .arg(principal_arg(
            OWNER_PRINCIPAL_ARG,
            "Allowance account owner principal",
        ))
        .arg(principal_arg(
            SPENDER_PRINCIPAL_ARG,
            "Allowance spender owner principal",
        ))
        .arg(subaccount_arg(
            OWNER_SUBACCOUNT_ARG,
            "Optional 32-byte owner account subaccount as hex",
        ))
        .arg(subaccount_arg(
            SPENDER_SUBACCOUNT_ARG,
            "Optional 32-byte spender account subaccount as hex",
        ));
    with_common_icrc_options(command)
}

pub(in crate::icrc) fn icrc_account_transaction_command() -> ClapCommand {
    ClapCommand::new("transaction")
        .bin_name("icq icrc account transaction")
        .about("Inspect live pages and complete cached account transaction history")
        .subcommand(icrc_account_transaction_page_command())
        .subcommand(icrc_account_transaction_list_command())
        .subcommand(icrc_account_transaction_refresh_command())
        .subcommand(icrc_account_transaction_cache_command())
}

pub(in crate::icrc) fn icrc_account_transaction_page_command() -> ClapCommand {
    let command = ClapCommand::new("page")
        .bin_name("icq icrc account transaction page")
        .about("Show an ICRC account transaction-history page from its index")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --start 100 --limit 25 --json\n  icq icrc account transaction page ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai",
        ))
        .args(account_transaction_target_args())
        .arg(
            value_arg(INDEX_CANISTER_ID_ARG)
                .long(INDEX_CANISTER_ID_ARG)
                .value_name("canister-id")
                .value_parser(super::principal_text_value_parser())
                .help("Explicit index canister; otherwise discover it from the ledger via ICRC-106"),
        )
        .arg(
            value_arg(START_ARG)
                .long(START_ARG)
                .value_name("block-index")
                .value_parser(account_transaction_cursor_value_parser())
                .help("Exclusive transaction id cursor returned as next_start by the prior page"),
        )
        .arg(
            value_arg(LIMIT_ARG)
                .long(LIMIT_ARG)
                .value_name("count")
                .default_value(DEFAULT_ICRC_TRANSACTIONS_LIMIT)
                .value_parser(
                    RangedU64ValueParser::<u32>::new().range(1..=MAX_ICRC_TRANSACTIONS_LIMIT),
                )
                .help("Maximum account transactions to request from the index"),
        );
    with_common_icrc_options(command)
}

pub(in crate::icrc) fn icrc_account_transaction_list_command() -> ClapCommand {
    let command = ClapCommand::new("list")
        .bin_name("icq icrc account transaction list")
        .about("List rows from a complete local ICRC account-history cache")
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            "Examples:\n  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --sort oldest --limit 100 --json",
        ))
        .args(account_transaction_target_args())
        .arg(
            value_arg(LIMIT_ARG)
                .long(LIMIT_ARG)
                .value_name("count")
                .default_value(DEFAULT_ICRC_TRANSACTIONS_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Maximum cached account transactions to return"),
        )
        .arg(
            value_arg(SORT_ARG)
                .long(SORT_ARG)
                .value_name("newest|oldest")
                .default_value("newest")
                .value_parser(["newest", "oldest"])
                .help("Cached transaction ordering"),
        );
    with_common_icrc_options(command)
}

pub(in crate::icrc) fn icrc_account_transaction_refresh_command() -> ClapCommand {
    let command = ClapCommand::new("refresh")
        .bin_name("icq icrc account transaction refresh")
        .about("Fetch and atomically cache complete ICRC account history")
        .after_help(collection_help(
            COLLECTION_MODE_FORCE_REFRESH,
            "Examples:\n  icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction refresh ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai --page-size 100 --json",
        ))
        .args(account_transaction_target_args())
        .arg(
            value_arg(INDEX_CANISTER_ID_ARG)
                .long(INDEX_CANISTER_ID_ARG)
                .value_name("canister-id")
                .value_parser(super::principal_text_value_parser())
                .help("Explicit index canister; otherwise discover it from the ledger via ICRC-106"),
        )
        .arg(
            value_arg(PAGE_SIZE_ARG)
                .long(PAGE_SIZE_ARG)
                .value_name("count")
                .default_value(DEFAULT_ICRC_ACCOUNT_TRANSACTION_PAGE_SIZE)
                .value_parser(RangedU64ValueParser::<u32>::new().range(
                    1..=u64::from(ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE),
                ))
                .help("Transactions requested per index page"),
        )
        .arg(
            value_arg(MAX_PAGES_ARG)
                .long(MAX_PAGES_ARG)
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Diagnostic page bound; reaching it fails without replacing the cache"),
        );
    with_common_icrc_options(command)
}

pub(in crate::icrc) fn icrc_account_transaction_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq icrc account transaction cache")
        .about("Inspect local complete account-history cache state")
        .subcommand(icrc_account_transaction_cache_status_command())
}

pub(in crate::icrc) fn icrc_account_transaction_cache_status_command() -> ClapCommand {
    let command = ClapCommand::new("status")
        .bin_name("icq icrc account transaction cache status")
        .about("Show local account-history cache and latest refresh-attempt status")
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            "Examples:\n  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa\n  icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --json",
        ))
        .args(account_transaction_target_args());
    with_common_icrc_options(command)
}

fn account_transaction_target_args() -> [clap::Arg; 3] {
    [
        ledger_canister_id_arg(),
        principal_arg(PRINCIPAL_ARG, "Account owner principal"),
        subaccount_arg(SUBACCOUNT_ARG, "Optional 32-byte ICRC subaccount as hex"),
    ]
}

fn account_transaction_cursor_value_parser() -> ValueParser {
    ValueParser::new(|value: &str| {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("expected unsigned decimal transaction id".to_string());
        }
        value
            .parse::<candid::Nat>()
            .map(|cursor| cursor.0.to_str_radix(10))
            .map_err(|error| error.to_string())
    })
}
