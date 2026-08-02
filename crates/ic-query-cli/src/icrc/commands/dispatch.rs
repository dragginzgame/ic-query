//! Module: icrc::commands::dispatch
//!
//! Responsibility: dispatch typed ICRC command matches into report requests.
//! Does not own: Clap command definitions, live calls, or report rendering.
//! Boundary: converts one composed parse tree into public requests and one output.

use super::{
    IcrcAccountTargetOptions, IcrcAccountTransactionCacheOptions,
    IcrcAccountTransactionListOptions, IcrcAccountTransactionPageOptions,
    IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions, IcrcArchivesOptions,
    IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
};
use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    icrc::IcrcCommandError,
    progress::StderrQueryProgress,
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::icrc::{
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    IcrcAccountTransactionCacheRequest, IcrcAccountTransactionListRequest,
    IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshRequest, IcrcAllowanceRequest,
    IcrcArchivesRequest, IcrcBalanceRequest, IcrcError, IcrcLedgerRequest, IcrcTransactionsRequest,
    build_icrc_account_transaction_cache_status_report, build_icrc_account_transaction_list_report,
    build_icrc_account_transaction_page_report, build_icrc_allowance_report,
    build_icrc_archives_report, build_icrc_balance_report, build_icrc_block_types_report,
    build_icrc_capabilities_report, build_icrc_index_report, build_icrc_tip_certificate_report,
    build_icrc_token_report, build_icrc_transactions_report,
    icrc_account_transaction_cache_status_report_text, icrc_account_transaction_list_report_text,
    icrc_account_transaction_page_report_text, icrc_account_transaction_refresh_report_text,
    icrc_allowance_report_text, icrc_archives_report_text, icrc_balance_report_text,
    icrc_block_types_report_text, icrc_capabilities_report_text, icrc_index_report_text,
    icrc_tip_certificate_report_text, icrc_token_report_text, icrc_transactions_report_text,
    refresh_icrc_account_transaction_cache_with_progress,
};
use serde::Serialize;

pub fn run_matches(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("ledger", matches)) => run_icrc_ledger(matches),
        Some(("account", matches)) => run_icrc_account(matches),
        _ => unreachable!("clap requires a known ICRC subcommand"),
    }
}

fn run_icrc_ledger(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("token", matches)) => {
            run_simple_ledger_report(matches, build_icrc_token_report, icrc_token_report_text)
        }
        Some(("index", matches)) => {
            run_simple_ledger_report(matches, build_icrc_index_report, icrc_index_report_text)
        }
        Some(("transactions", matches)) => run_icrc_transactions(matches),
        Some(("block-types", matches)) => run_simple_ledger_report(
            matches,
            build_icrc_block_types_report,
            icrc_block_types_report_text,
        ),
        Some(("archives", matches)) => run_icrc_archives(matches),
        Some(("tip-certificate", matches)) => run_simple_ledger_report(
            matches,
            build_icrc_tip_certificate_report,
            icrc_tip_certificate_report_text,
        ),
        Some(("capabilities", matches)) => run_simple_ledger_report(
            matches,
            build_icrc_capabilities_report,
            icrc_capabilities_report_text,
        ),
        _ => unreachable!("clap requires a known ICRC ledger subcommand"),
    }
}

fn run_simple_ledger_report<Report>(
    matches: &ArgMatches,
    build: fn(&IcrcLedgerRequest) -> Result<Report, IcrcError>,
    render_text: fn(&Report) -> String,
) -> Result<(), IcrcCommandError>
where
    Report: Serialize,
{
    let options = IcrcLedgerOptions::from_matches(matches);
    let request = IcrcLedgerRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build(&request)?;
    write_text_or_json(options.format, &report, render_text)
}

fn run_icrc_account(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("balance", matches)) => run_icrc_balance(matches),
        Some(("allowance", matches)) => run_icrc_allowance(matches),
        Some(("transaction", matches)) => run_icrc_account_transaction(matches),
        _ => unreachable!("clap requires a known ICRC account subcommand"),
    }
}

fn run_icrc_balance(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcBalanceOptions::from_matches(matches);
    let ledger = options.ledger;
    let request = IcrcBalanceRequest {
        source_endpoint: ledger.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: ledger.ledger_canister_id,
        account_owner: options.account_owner,
        subaccount_hex: options.subaccount_hex,
    };
    let report = build_icrc_balance_report(&request)?;
    write_text_or_json(ledger.format, &report, icrc_balance_report_text)
}

fn run_icrc_allowance(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAllowanceOptions::from_matches(matches);
    let ledger = options.ledger;
    let request = IcrcAllowanceRequest {
        source_endpoint: ledger.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: ledger.ledger_canister_id,
        account_owner: options.account_owner,
        account_subaccount_hex: options.account_subaccount_hex,
        spender_owner: options.spender_owner,
        spender_subaccount_hex: options.spender_subaccount_hex,
    };
    let report = build_icrc_allowance_report(&request)?;
    write_text_or_json(ledger.format, &report, icrc_allowance_report_text)
}

fn run_icrc_account_transaction(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("page", matches)) => run_icrc_account_transaction_page(matches),
        Some(("list", matches)) => run_icrc_account_transaction_list(matches),
        Some(("refresh", matches)) => run_icrc_account_transaction_refresh(matches),
        Some(("cache", matches)) => run_icrc_account_transaction_cache(matches),
        _ => unreachable!("clap requires a known ICRC account transaction subcommand"),
    }
}

fn run_icrc_account_transaction_page(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAccountTransactionPageOptions::from_matches(matches);
    let target = options.target;
    let request = IcrcAccountTransactionPageRequest {
        source_endpoint: target.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: target.ledger_canister_id,
        index_canister_id: options.index_canister_id,
        account_owner: target.account_owner,
        subaccount_hex: target.subaccount_hex,
        start: options.start,
        limit: options.limit,
    };
    let report = build_icrc_account_transaction_page_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        icrc_account_transaction_page_report_text,
    )
}

fn run_icrc_account_transaction_list(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAccountTransactionListOptions::from_matches(matches);
    let request = IcrcAccountTransactionListRequest {
        cache: account_transaction_cache_request(options.target)?,
        limit: options.limit,
        sort: options.sort,
    };
    let report = build_icrc_account_transaction_list_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        icrc_account_transaction_list_report_text,
    )
}

fn run_icrc_account_transaction_refresh(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAccountTransactionRefreshOptions::from_matches(matches);
    let request = IcrcAccountTransactionRefreshRequest {
        cache: account_transaction_cache_request(options.target)?,
        now_unix_secs: current_unix_secs()?,
        index_canister_id: options.index_canister_id,
        page_size: options.page_size,
        max_pages: options.max_pages,
        lock_stale_after_seconds: DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    };
    let mut progress = StderrQueryProgress::new();
    let report = refresh_icrc_account_transaction_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(
        options.format,
        &report,
        icrc_account_transaction_refresh_report_text,
    )
}

fn run_icrc_account_transaction_cache(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => run_icrc_account_transaction_cache_status(matches),
        _ => unreachable!("clap requires a known ICRC account transaction cache subcommand"),
    }
}

fn run_icrc_account_transaction_cache_status(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAccountTransactionCacheOptions::from_matches(matches);
    let request = account_transaction_cache_request(options.target)?;
    let report = build_icrc_account_transaction_cache_status_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        icrc_account_transaction_cache_status_report_text,
    )
}

fn account_transaction_cache_request(
    target: IcrcAccountTargetOptions,
) -> Result<IcrcAccountTransactionCacheRequest, IcrcCommandError> {
    Ok(IcrcAccountTransactionCacheRequest {
        cache_root: cache_root().map_err(|error| IcrcCommandError::Usage(error.to_string()))?,
        source_endpoint: target.source_endpoint,
        ledger_canister_id: target.ledger_canister_id,
        account_owner: target.account_owner,
        subaccount_hex: target.subaccount_hex,
    })
}

fn run_icrc_transactions(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcTransactionsOptions::from_matches(matches);
    let ledger = options.ledger;
    let request = IcrcTransactionsRequest {
        source_endpoint: ledger.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: ledger.ledger_canister_id,
        start: options.start,
        limit: options.limit,
        follow_archives: options.follow_archives,
    };
    let report = build_icrc_transactions_report(&request)?;
    write_text_or_json(ledger.format, &report, icrc_transactions_report_text)
}

fn run_icrc_archives(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcArchivesOptions::from_matches(matches);
    let ledger = options.ledger;
    let request = IcrcArchivesRequest {
        source_endpoint: ledger.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: ledger.ledger_canister_id,
        from_canister_id: options.from_canister_id,
    };
    let report = build_icrc_archives_report(&request)?;
    write_text_or_json(ledger.format, &report, icrc_archives_report_text)
}
