//! Module: icrc::commands
//!
//! Responsibility: parse and run generic ICRC CLI commands.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: maps clap options into report requests and writes text/JSON output.

mod dispatch;
mod options;
#[cfg(test)]
pub(in crate::icrc) mod test_support;

pub use dispatch::run_matches;
use options::{
    IcrcAccountTargetOptions, IcrcAccountTransactionCacheOptions,
    IcrcAccountTransactionListOptions, IcrcAccountTransactionPageOptions,
    IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions, IcrcArchivesOptions,
    IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
};

#[cfg(test)]
use crate::cli::clap::render_help;
use crate::cli::{
    clap::{flag_arg, required_string, value_arg},
    common::{
        COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_FORCE_REFRESH, COLLECTION_MODE_LIVE,
        OutputFormat, collection_help, json_arg, output_format, source_endpoint_arg,
    },
};
use candid::Principal;
use clap::{
    ArgMatches, Command as ClapCommand,
    builder::{RangedU64ValueParser, ValueParser},
};
use ic_query::icrc::{
    DEFAULT_ICRC_SOURCE_ENDPOINT, ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, normalize_subaccount_hex,
};

const DEFAULT_ICRC_TRANSACTIONS_LIMIT: &str = "25";
const MAX_ICRC_TRANSACTIONS_LIMIT: u64 = 100;
const DEFAULT_ICRC_ACCOUNT_TRANSACTION_PAGE_SIZE: &str = "100";
const LEDGER_CANISTER_ID_ARG: &str = "ledger-canister-id";
const INDEX_CANISTER_ID_ARG: &str = "index-canister-id";
const PRINCIPAL_ARG: &str = "principal";
const OWNER_PRINCIPAL_ARG: &str = "owner-principal";
const SPENDER_PRINCIPAL_ARG: &str = "spender-principal";
const SUBACCOUNT_ARG: &str = "subaccount";
const OWNER_SUBACCOUNT_ARG: &str = "owner-subaccount";
const SPENDER_SUBACCOUNT_ARG: &str = "spender-subaccount";
const START_ARG: &str = "start";
const LIMIT_ARG: &str = "limit";
const PAGE_SIZE_ARG: &str = "page-size";
const MAX_PAGES_ARG: &str = "max-pages";
const SORT_ARG: &str = "sort";
const FOLLOW_ARCHIVES_ARG: &str = "follow-archives";
const FROM_CANISTER_ID_ARG: &str = "from";
const SOURCE_ENDPOINT_ARG: &str = "source-endpoint";
const ICRC_SOURCE_ENDPOINT_HELP: &str = "IC API endpoint used for ICRC ledger queries";

pub fn command() -> ClapCommand {
    ClapCommand::new("icrc")
        .bin_name("icq icrc")
        .about("Inspect generic ICRC ledgers")
        .subcommand_required(true)
        .subcommand(icrc_ledger_command())
        .subcommand(icrc_account_command())
}

fn icrc_ledger_command() -> ClapCommand {
    ClapCommand::new("ledger")
        .bin_name("icq icrc ledger")
        .about("Inspect ledger-wide ICRC metadata and transactions")
        .subcommand_required(true)
        .subcommand(icrc_capabilities_command())
        .subcommand(icrc_token_command())
        .subcommand(icrc_index_command())
        .subcommand(icrc_transactions_command())
        .subcommand(icrc_block_types_command())
        .subcommand(icrc_archives_command())
        .subcommand(icrc_tip_certificate_command())
}

fn icrc_account_command() -> ClapCommand {
    ClapCommand::new("account")
        .bin_name("icq icrc account")
        .about("Inspect ICRC account balances, allowances, and transaction history")
        .subcommand_required(true)
        .subcommand(icrc_balance_command())
        .subcommand(icrc_allowance_command())
        .subcommand(icrc_account_transaction_command())
}

fn icrc_token_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "token",
        "icq icrc ledger token",
        "Show generic ICRC token metadata by ledger canister id",
        "Examples:\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

fn icrc_capabilities_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "capabilities",
        "icq icrc ledger capabilities",
        "Probe generic ICRC ledger endpoint capabilities",
        "Examples:\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

fn icrc_balance_command() -> ClapCommand {
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

fn icrc_allowance_command() -> ClapCommand {
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

fn icrc_account_transaction_command() -> ClapCommand {
    ClapCommand::new("transaction")
        .bin_name("icq icrc account transaction")
        .about("Inspect live pages and complete cached account transaction history")
        .subcommand_required(true)
        .subcommand(icrc_account_transaction_page_command())
        .subcommand(icrc_account_transaction_list_command())
        .subcommand(icrc_account_transaction_refresh_command())
        .subcommand(icrc_account_transaction_cache_command())
}

fn icrc_account_transaction_page_command() -> ClapCommand {
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
                .value_parser(principal_text_value_parser())
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

fn icrc_account_transaction_list_command() -> ClapCommand {
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

fn icrc_account_transaction_refresh_command() -> ClapCommand {
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
                .value_parser(principal_text_value_parser())
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

fn icrc_account_transaction_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq icrc account transaction cache")
        .about("Inspect local complete account-history cache state")
        .subcommand_required(true)
        .subcommand(icrc_account_transaction_cache_status_command())
}

fn icrc_account_transaction_cache_status_command() -> ClapCommand {
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

fn icrc_index_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "index",
        "icq icrc ledger index",
        "Show a generic ICRC ledger index canister",
        "Examples:\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

fn icrc_transactions_command() -> ClapCommand {
    let command = ClapCommand::new("transactions")
        .bin_name("icq icrc ledger transactions")
        .about("Show a generic ICRC ledger transaction history page")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --json",
        ))
        .arg(ledger_canister_id_arg())
        .arg(
            value_arg(START_ARG)
                .long(START_ARG)
                .value_name("index")
                .default_value("0")
                .value_parser(clap::value_parser!(u64))
                .help("First ICRC-3 block index to request from the ledger"),
        )
        .arg(
            value_arg(LIMIT_ARG)
                .long(LIMIT_ARG)
                .value_name("count")
                .default_value(DEFAULT_ICRC_TRANSACTIONS_LIMIT)
                .value_parser(
                    RangedU64ValueParser::<u32>::new().range(1..=MAX_ICRC_TRANSACTIONS_LIMIT),
                )
                .help("Maximum ICRC-3 blocks to request from the ledger"),
        );
    let command = with_icrc_source_endpoint_option(command).arg(
        flag_arg(FOLLOW_ARCHIVES_ARG)
            .long(FOLLOW_ARCHIVES_ARG)
            .help("Follow returned ICRC-3 archive callbacks for the requested block page"),
    );
    with_icrc_json_option(command)
}

fn icrc_block_types_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "block-types",
        "icq icrc ledger block-types",
        "Show generic ICRC-3 ledger supported block types",
        "Examples:\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

fn icrc_archives_command() -> ClapCommand {
    let command = ClapCommand::new("archives")
        .bin_name("icq icrc ledger archives")
        .about("Show generic ICRC-3 ledger archive ranges")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --json",
        ))
        .arg(ledger_canister_id_arg())
        .arg(
            value_arg(FROM_CANISTER_ID_ARG)
                .long(FROM_CANISTER_ID_ARG)
                .value_name("canister-id")
                .value_parser(principal_text_value_parser())
                .help("Last archive canister already seen; returns later archives"),
        );
    with_common_icrc_options(command)
}

fn icrc_tip_certificate_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "tip-certificate",
        "icq icrc ledger tip-certificate",
        "Show a generic ICRC-3 ledger tip certificate",
        "Examples:\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

fn simple_icrc_ledger_command(
    name: &'static str,
    bin_name: &'static str,
    about: &'static str,
    examples: &'static str,
) -> ClapCommand {
    let command = ClapCommand::new(name)
        .bin_name(bin_name)
        .about(about)
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

#[cfg(test)]
fn usage() -> String {
    render_help(command())
}

#[cfg(test)]
fn icrc_ledger_usage() -> String {
    render_help(icrc_ledger_command())
}

#[cfg(test)]
fn icrc_account_usage() -> String {
    render_help(icrc_account_command())
}

#[cfg(test)]
fn icrc_token_usage() -> String {
    render_help(icrc_token_command())
}

#[cfg(test)]
fn icrc_capabilities_usage() -> String {
    render_help(icrc_capabilities_command())
}

#[cfg(test)]
fn icrc_balance_usage() -> String {
    render_help(icrc_balance_command())
}

#[cfg(test)]
fn icrc_allowance_usage() -> String {
    render_help(icrc_allowance_command())
}

#[cfg(test)]
fn icrc_account_transaction_usage() -> String {
    render_help(icrc_account_transaction_command())
}

#[cfg(test)]
fn icrc_account_transaction_page_usage() -> String {
    render_help(icrc_account_transaction_page_command())
}

#[cfg(test)]
fn icrc_account_transaction_list_usage() -> String {
    render_help(icrc_account_transaction_list_command())
}

#[cfg(test)]
fn icrc_account_transaction_refresh_usage() -> String {
    render_help(icrc_account_transaction_refresh_command())
}

#[cfg(test)]
fn icrc_account_transaction_cache_usage() -> String {
    render_help(icrc_account_transaction_cache_command())
}

#[cfg(test)]
fn icrc_account_transaction_cache_status_usage() -> String {
    render_help(icrc_account_transaction_cache_status_command())
}

#[cfg(test)]
fn icrc_index_usage() -> String {
    render_help(icrc_index_command())
}

#[cfg(test)]
fn icrc_transactions_usage() -> String {
    render_help(icrc_transactions_command())
}

#[cfg(test)]
fn icrc_block_types_usage() -> String {
    render_help(icrc_block_types_command())
}

#[cfg(test)]
fn icrc_archives_usage() -> String {
    render_help(icrc_archives_command())
}

#[cfg(test)]
fn icrc_tip_certificate_usage() -> String {
    render_help(icrc_tip_certificate_command())
}

fn ledger_canister_id_arg() -> clap::Arg {
    principal_arg(LEDGER_CANISTER_ID_ARG, "ICRC ledger canister principal")
}

fn account_transaction_target_args() -> [clap::Arg; 3] {
    [
        ledger_canister_id_arg(),
        principal_arg(PRINCIPAL_ARG, "Account owner principal"),
        subaccount_arg(SUBACCOUNT_ARG, "Optional 32-byte ICRC subaccount as hex"),
    ]
}

fn with_common_icrc_options(command: ClapCommand) -> ClapCommand {
    with_icrc_json_option(with_icrc_source_endpoint_option(command))
}

fn with_icrc_source_endpoint_option(command: ClapCommand) -> ClapCommand {
    command.arg(icrc_source_endpoint_arg())
}

fn with_icrc_json_option(command: ClapCommand) -> ClapCommand {
    command.arg(json_arg())
}

fn icrc_source_endpoint_arg() -> clap::Arg {
    source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT).help(ICRC_SOURCE_ENDPOINT_HELP)
}

fn principal_arg(id: &'static str, help: &'static str) -> clap::Arg {
    value_arg(id)
        .value_name(id)
        .required(true)
        .value_parser(principal_text_value_parser())
        .help(help)
}

fn subaccount_arg(id: &'static str, help: &'static str) -> clap::Arg {
    value_arg(id)
        .long(id)
        .value_name("hex")
        .value_parser(subaccount_hex_value_parser())
        .help(help)
}

fn format_from_matches(matches: &ArgMatches) -> OutputFormat {
    output_format(matches)
}

fn source_endpoint_from_matches(matches: &ArgMatches) -> String {
    required_string(matches, SOURCE_ENDPOINT_ARG)
}

fn principal_text_value_parser() -> ValueParser {
    ValueParser::new(|value: &str| {
        Principal::from_text(value)
            .map(|principal| principal.to_text())
            .map_err(|err| err.to_string())
    })
}

fn subaccount_hex_value_parser() -> ValueParser {
    ValueParser::new(|value: &str| normalize_subaccount_hex(value).map_err(|err| err.to_string()))
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
