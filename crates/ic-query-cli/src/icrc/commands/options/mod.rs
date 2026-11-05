//! Module: icrc::commands::options
//!
//! Responsibility: parse Clap matches into ICRC command options and public requests.
//! Does not own: command dispatch, report construction, or output.
//! Boundary: carries parsed values into requests without report construction or IO.

use super::{
    ACCOUNT_ID_ARG, DEFAULT_ICRC_ACCOUNT_TRANSACTION_PAGE_SIZE, DEFAULT_ICRC_TRANSACTIONS_LIMIT,
    FOLLOW_ARCHIVES_ARG, FROM_CANISTER_ID_ARG, INDEX_CANISTER_ID_ARG, LEDGER_CANISTER_ID_ARG,
    LIMIT_ARG, MAX_ICRC_TRANSACTIONS_LIMIT, MAX_PAGES_ARG, OWNER_PRINCIPAL_ARG,
    OWNER_SUBACCOUNT_ARG, PAGE_SIZE_ARG, PRINCIPAL_ARG, SORT_ARG, SPENDER_PRINCIPAL_ARG,
    SPENDER_SUBACCOUNT_ARG, START_ARG, SUBACCOUNT_ARG, principal_text_value_parser,
    subaccount_hex_value_parser,
};
use crate::cli::{
    clap::{required_typed, string_option},
    common::{OutputFormat, SourceOutputOptions},
};
use clap::{ArgMatches, Args, FromArgMatches, builder::RangedU64ValueParser};
use ic_query::icrc::IcrcAccountTransactionCacheRequest;
use ic_query::icrc::IcrcAccountTransactionListRequest;
use ic_query::icrc::IcrcAccountTransactionPageRequest;
use ic_query::icrc::IcrcAccountTransactionRefreshRequest;
use ic_query::icrc::IcrcAccountTransactionSort;
use ic_query::icrc::IcrcAllowanceRequest;
use ic_query::icrc::IcrcArchivesRequest;
use ic_query::icrc::IcrcTransactionsRequest;

mod analytics;
pub(in crate::icrc) use analytics::*;

///
/// IcrcLedgerOptions
///
/// Shared ledger target, output, and endpoint options for live ICRC queries.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::icrc) struct IcrcLedgerOptions {
    #[arg(id = LEDGER_CANISTER_ID_ARG, value_name = LEDGER_CANISTER_ID_ARG, value_parser = principal_text_value_parser(), help = "ICRC ledger canister principal")]
    pub(in crate::icrc) ledger_canister_id: String,
    #[arg(
        long,
        display_order = 1001,
        help = "Print raw JSON instead of human-readable text"
    )]
    json: bool,
    #[arg(id = "source-endpoint", long = "source-endpoint", display_order = 1000, value_name = "url", default_value = ic_query::icrc::DEFAULT_ICRC_SOURCE_ENDPOINT, help = "IC API endpoint used for ICRC ledger queries")]
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcLedgerOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        parsed_options(matches)
    }

    pub(in crate::icrc) const fn format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        }
    }

    pub(in crate::icrc) fn into_dashboard_request(
        self,
        now_unix_secs: u64,
    ) -> (OutputFormat, ic_query::ic::IcDashboardRequest, String) {
        let (format, request) = SourceOutputOptions {
            format: self.format(),
            source_endpoint: self.source_endpoint,
        }
        .into_dashboard_request(now_unix_secs);
        (format, request, self.ledger_canister_id)
    }

    pub(in crate::icrc) fn into_ledger_request(
        self,
        now_unix_secs: u64,
    ) -> (OutputFormat, ic_query::icrc::IcrcLedgerRequest) {
        (
            self.format(),
            ic_query::icrc::IcrcLedgerRequest::new(
                self.source_endpoint,
                now_unix_secs,
                self.ledger_canister_id,
            ),
        )
    }
}

///
/// IcrcAccountOptions
///
/// Shared ledger, output, endpoint and account identity for ICRC account commands.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::icrc) struct IcrcAccountOptions {
    #[command(flatten)]
    pub(in crate::icrc) ledger: IcrcLedgerOptions,
    #[arg(id = PRINCIPAL_ARG, value_name = PRINCIPAL_ARG, value_parser = principal_text_value_parser(), help = "Account owner principal")]
    pub(in crate::icrc) account_owner: String,
    #[arg(id = SUBACCOUNT_ARG, long = SUBACCOUNT_ARG, value_name = "hex", value_parser = subaccount_hex_value_parser(), help = "Optional 32-byte ICRC subaccount as hex")]
    pub(in crate::icrc) subaccount_hex: Option<String>,
}

impl IcrcAccountOptions {
    pub(in crate::icrc) fn from_matches(matches: &ArgMatches) -> Self {
        parsed_options(matches)
    }

    pub(in crate::icrc) fn into_cache_request(
        self,
        cache_root: std::path::PathBuf,
    ) -> (OutputFormat, IcrcAccountTransactionCacheRequest) {
        (
            self.ledger.format(),
            IcrcAccountTransactionCacheRequest {
                cache_root,
                source_endpoint: self.ledger.source_endpoint,
                ledger_canister_id: self.ledger.ledger_canister_id,
                account_owner: self.account_owner,
                subaccount_hex: self.subaccount_hex,
            },
        )
    }
}

///
/// IcrcAllowanceOptions
///
/// Typed account identities for one allowance query.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct IcrcAllowanceOptions {
    #[command(flatten)]
    ledger: IcrcLedgerOptions,
    #[arg(id = OWNER_PRINCIPAL_ARG, value_name = OWNER_PRINCIPAL_ARG, value_parser = principal_text_value_parser(), help = "Allowance account owner principal")]
    owner: String,
    #[arg(id = SPENDER_PRINCIPAL_ARG, value_name = SPENDER_PRINCIPAL_ARG, value_parser = principal_text_value_parser(), help = "Allowance spender owner principal")]
    spender: String,
    #[arg(id = OWNER_SUBACCOUNT_ARG, long = OWNER_SUBACCOUNT_ARG, value_name = "hex", value_parser = subaccount_hex_value_parser(), help = "Optional 32-byte owner account subaccount as hex")]
    owner_subaccount: Option<String>,
    #[arg(id = SPENDER_SUBACCOUNT_ARG, long = SPENDER_SUBACCOUNT_ARG, value_name = "hex", value_parser = subaccount_hex_value_parser(), help = "Optional 32-byte spender account subaccount as hex")]
    spender_subaccount: Option<String>,
}

///
/// IcrcTransactionPageOptions
///
/// Typed bounds for one live account-history page.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct IcrcTransactionPageOptions {
    #[command(flatten)]
    account: IcrcAccountOptions,
    #[arg(id = INDEX_CANISTER_ID_ARG, long = INDEX_CANISTER_ID_ARG, value_name = "canister-id", value_parser = principal_text_value_parser(), help = "Explicit index canister; otherwise discover it from the ledger via ICRC-106")]
    index: Option<String>,
    #[arg(id = START_ARG, long = START_ARG, value_name = "block-index", value_parser = account_transaction_cursor, help = "Exclusive transaction id cursor returned as next_start by the prior page")]
    start: Option<String>,
    #[arg(id = LIMIT_ARG, long = LIMIT_ARG, value_name = "count", default_value = DEFAULT_ICRC_TRANSACTIONS_LIMIT, value_parser = RangedU64ValueParser::<u32>::new().range(1..=MAX_ICRC_TRANSACTIONS_LIMIT), help = "Maximum account transactions to request from the index")]
    limit: u32,
}

///
/// IcrcTransactionListOptions
///
/// Typed view of an already complete local account history.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct IcrcTransactionListOptions {
    #[command(flatten)]
    account: IcrcAccountOptions,
    #[arg(id = LIMIT_ARG, long = LIMIT_ARG, value_name = "count", default_value = DEFAULT_ICRC_TRANSACTIONS_LIMIT, value_parser = RangedU64ValueParser::<u32>::new().range(1..), help = "Maximum cached account transactions to return")]
    limit: u32,
    #[arg(id = SORT_ARG, long = SORT_ARG, value_name = "newest|oldest", default_value = "newest", value_parser = crate::cli::clap::choice_parser(IcrcAccountTransactionSort::ALL, IcrcAccountTransactionSort::as_str), help = "Cached transaction ordering")]
    sort: IcrcAccountTransactionSort,
}

///
/// IcrcTransactionRefreshOptions
///
/// Typed collection budget for an explicit account-history refresh.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct IcrcTransactionRefreshOptions {
    #[command(flatten)]
    account: IcrcAccountOptions,
    #[arg(id = INDEX_CANISTER_ID_ARG, long = INDEX_CANISTER_ID_ARG, value_name = "canister-id", value_parser = principal_text_value_parser(), help = "Explicit index canister; otherwise discover it from the ledger via ICRC-106")]
    index: Option<String>,
    #[arg(id = PAGE_SIZE_ARG, long = PAGE_SIZE_ARG, value_name = "count", default_value = DEFAULT_ICRC_ACCOUNT_TRANSACTION_PAGE_SIZE, value_parser = RangedU64ValueParser::<u32>::new().range(1..=u64::from(ic_query::icrc::ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE)), help = "Transactions requested per index page")]
    page_size: u32,
    #[arg(id = MAX_PAGES_ARG, long = MAX_PAGES_ARG, value_name = "count", value_parser = RangedU64ValueParser::<u32>::new().range(1..), help = "Diagnostic page bound; reaching it fails without replacing the cache")]
    max_pages: Option<u32>,
}

fn parsed_options<Options: FromArgMatches>(matches: &ArgMatches) -> Options {
    Options::from_arg_matches(matches).expect("Clap admitted the typed account argument schema")
}

fn account_transaction_cursor(value: &str) -> Result<String, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected unsigned decimal transaction id".to_string());
    }
    value
        .parse::<ic_platform::candid::contract::Nat>()
        .map(|cursor| cursor.0.to_str_radix(10))
        .map_err(|error| error.to_string())
}

pub(in crate::icrc) fn allowance_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAllowanceRequest) {
    let options: IcrcAllowanceOptions = parsed_options(matches);
    let (format, ledger) = options.ledger.into_ledger_request(now_unix_secs);
    (
        format,
        IcrcAllowanceRequest {
            ledger,
            account_owner: options.owner,
            account_subaccount_hex: options.owner_subaccount,
            spender_owner: options.spender,
            spender_subaccount_hex: options.spender_subaccount,
        },
    )
}

pub(in crate::icrc) fn account_transaction_page_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAccountTransactionPageRequest) {
    let options: IcrcTransactionPageOptions = parsed_options(matches);
    let (format, ledger) = options.account.ledger.into_ledger_request(now_unix_secs);
    (
        format,
        IcrcAccountTransactionPageRequest {
            ledger,
            index_canister_id: options.index,
            account_owner: options.account.account_owner,
            subaccount_hex: options.account.subaccount_hex,
            start: options.start,
            limit: options.limit,
        },
    )
}

pub(in crate::icrc) fn account_transaction_list_request(
    matches: &ArgMatches,
    cache_root: std::path::PathBuf,
) -> (OutputFormat, IcrcAccountTransactionListRequest) {
    let options: IcrcTransactionListOptions = parsed_options(matches);
    let (format, cache) = options.account.into_cache_request(cache_root);
    (
        format,
        IcrcAccountTransactionListRequest {
            cache,
            limit: options.limit,
            sort: options.sort,
        },
    )
}

pub(in crate::icrc) fn account_transaction_refresh_request(
    matches: &ArgMatches,
    cache_root: std::path::PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcAccountTransactionRefreshRequest) {
    let options: IcrcTransactionRefreshOptions = parsed_options(matches);
    let (format, cache) = options.account.into_cache_request(cache_root);
    (
        format,
        IcrcAccountTransactionRefreshRequest {
            cache,
            now_unix_secs,
            index_canister_id: options.index,
            page_size: options.page_size,
            max_pages: options.max_pages,
        },
    )
}

pub(in crate::icrc) fn transactions_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcTransactionsRequest) {
    let (format, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_ledger_request(now_unix_secs);
    (
        format,
        IcrcTransactionsRequest {
            ledger,
            start: required_typed(matches, START_ARG),
            limit: required_typed(matches, LIMIT_ARG),
            follow_archives: matches.get_flag(FOLLOW_ARCHIVES_ARG),
        },
    )
}

pub(in crate::icrc) fn archives_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcrcArchivesRequest) {
    let (format, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_ledger_request(now_unix_secs);
    (
        format,
        IcrcArchivesRequest {
            ledger,
            from_canister_id: string_option(matches, FROM_CANISTER_ID_ARG),
        },
    )
}

#[cfg(test)]
mod tests;
