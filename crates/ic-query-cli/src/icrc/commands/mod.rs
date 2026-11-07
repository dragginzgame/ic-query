//! Module: icrc::commands
//!
//! Responsibility: compose generic ICRC CLI command families and shared arguments.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: keeps ledger and account grammar separate behind one ICRC facade.

mod account;
mod analytics;
mod dispatch;
mod ledger;
pub(in crate::icrc) mod options;
#[cfg(test)]
pub(in crate::icrc) use account::{
    command as icrc_account_command, icrc_account_transaction_cache_command,
    icrc_account_transaction_cache_status_command, icrc_account_transaction_command,
    icrc_account_transaction_list_command, icrc_account_transaction_page_command,
    icrc_account_transaction_refresh_command, icrc_allowance_command, icrc_balance_command,
};
#[cfg(test)]
pub(in crate::icrc) use analytics::{
    command as icrc_analytics_command, icrc_analytics_account_info_command,
    icrc_analytics_account_list_command, icrc_analytics_holder_list_command,
    icrc_analytics_token_values_command, icrc_analytics_total_supply_command,
};
#[cfg(test)]
pub(in crate::icrc) use ledger::{
    command as icrc_ledger_command, icrc_archives_command, icrc_block_types_command,
    icrc_capabilities_command, icrc_index_command, icrc_tip_certificate_command,
    icrc_token_command, icrc_transactions_command,
};

pub use dispatch::run_matches;
pub(in crate::icrc) use options::{IcrcAccountOptions, IcrcLedgerOptions};

use crate::cli::clap::principal_text_value_parser;
use clap::{Command as ClapCommand, builder::ValueParser};
use ic_query::icrc::normalize_subaccount_hex;

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
const ACCOUNT_ID_ARG: &str = "account-id";
const PAGE_SIZE_ARG: &str = "page-size";
const MAX_PAGES_ARG: &str = "max-pages";
const SORT_ARG: &str = "sort";
const FOLLOW_ARCHIVES_ARG: &str = "follow-archives";
const FROM_CANISTER_ID_ARG: &str = "from";

fn with_ledger_options<T: clap::Args>(command: ClapCommand) -> ClapCommand {
    crate::cli::clap::with_options::<T>(command)
        .mut_arg("source-endpoint", |arg| {
            arg.display_order(1000)
                .help("IC API endpoint used for ICRC ledger queries")
        })
        .mut_arg("json", |arg| arg.display_order(1001))
}

pub fn command() -> ClapCommand {
    ClapCommand::new("icrc")
        .about("Inspect generic ICRC ledgers and official analytics")
        .subcommand(ledger::command())
        .subcommand(account::command())
        .subcommand(analytics::command())
}

fn subaccount_hex_value_parser() -> ValueParser {
    ValueParser::new(|value: &str| normalize_subaccount_hex(value).map_err(|err| err.to_string()))
}
