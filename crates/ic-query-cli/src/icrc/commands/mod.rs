//! Module: icrc::commands
//!
//! Responsibility: compose generic ICRC CLI command families and shared arguments.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: keeps ledger and account grammar separate behind one ICRC facade.

mod account;
mod dispatch;
mod ledger;
mod options;
#[cfg(test)]
pub(in crate::icrc) use account::{
    command as icrc_account_command, icrc_account_transaction_cache_command,
    icrc_account_transaction_cache_status_command, icrc_account_transaction_command,
    icrc_account_transaction_list_command, icrc_account_transaction_page_command,
    icrc_account_transaction_refresh_command, icrc_allowance_command, icrc_balance_command,
};
#[cfg(test)]
pub(in crate::icrc) use ledger::{
    command as icrc_ledger_command, icrc_archives_command, icrc_block_types_command,
    icrc_capabilities_command, icrc_index_command, icrc_tip_certificate_command,
    icrc_token_command, icrc_transactions_command,
};

pub use dispatch::run_matches;
pub(in crate::icrc) use options::{
    IcrcAccountTargetOptions, IcrcAccountTransactionCacheOptions,
    IcrcAccountTransactionListOptions, IcrcAccountTransactionPageOptions,
    IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions, IcrcArchivesOptions,
    IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
};

use crate::cli::{
    clap::{required_string, value_arg},
    common::{OutputFormat, json_arg, output_format, source_endpoint_arg},
};
use candid::Principal;
use clap::{ArgMatches, Command as ClapCommand, builder::ValueParser};
use ic_query::icrc::{DEFAULT_ICRC_SOURCE_ENDPOINT, normalize_subaccount_hex};

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
        .subcommand(ledger::command())
        .subcommand(account::command())
}

fn ledger_canister_id_arg() -> clap::Arg {
    principal_arg(LEDGER_CANISTER_ID_ARG, "ICRC ledger canister principal")
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
