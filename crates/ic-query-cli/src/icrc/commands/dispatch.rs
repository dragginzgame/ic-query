//! Module: icrc::commands::dispatch
//!
//! Responsibility: dispatch typed ICRC command matches into report requests.
//! Does not own: Clap command definitions, live calls, or report rendering.
//! Boundary: converts one composed parse tree into public requests and one output.

use super::options::{
    account_transaction_list_request, account_transaction_page_request,
    account_transaction_refresh_request, allowance_request, analytics_account_info_request,
    analytics_account_list_request, analytics_holder_list_request, analytics_token_value_request,
    analytics_total_supply_request, archives_request, transactions_request,
};

use super::{IcrcAccountOptions, IcrcLedgerOptions};
use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    icrc::IcrcCommandError,
    progress::StderrQueryProgress,
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::ic::{
    IcIcrcIndexedCountKind, IcIcrcIndexedCountRequest, build_icrc_account_info_report,
    build_icrc_account_list_report, build_icrc_holder_list_report, build_icrc_indexed_count_report,
    build_icrc_token_value_report, build_icrc_total_supply_report, icrc_account_info_report_text,
    icrc_account_list_report_text, icrc_holder_list_report_text, icrc_indexed_count_report_text,
    icrc_token_value_report_text, icrc_total_supply_report_text,
};
use ic_query::icrc::{
    IcrcBalanceRequest, IcrcError, IcrcLedgerRequest,
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

pub async fn run_matches(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("ledger", matches)) => run_icrc_ledger(matches).await,
        Some(("account", matches)) => run_icrc_account(matches).await,
        Some(("analytics", matches)) => run_icrc_analytics(matches).await,
        _ => unreachable!("clap requires a known ICRC subcommand"),
    }
}

async fn run_icrc_analytics(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("account", matches)) => run_icrc_analytics_account(matches).await,
        Some(("holder", matches)) => run_icrc_analytics_holder(matches).await,
        Some(("token-values", matches)) => run_icrc_analytics_token_values(matches).await,
        Some(("total-supply", matches)) => run_icrc_analytics_total_supply(matches).await,
        Some(("transaction", matches)) => match matches.subcommand() {
            Some(("count", matches)) => {
                run_icrc_analytics_indexed_count_leaf(matches, IcIcrcIndexedCountKind::Transaction)
                    .await
            }
            _ => unreachable!("clap requires a known ICRC indexed-count subcommand"),
        },
        _ => unreachable!("clap requires a known ICRC analytics subcommand"),
    }
}

async fn run_icrc_analytics_account(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("count", matches)) => {
            run_icrc_analytics_indexed_count_leaf(matches, IcIcrcIndexedCountKind::Account).await
        }
        Some(("info", matches)) => {
            let (format, request) = analytics_account_info_request(matches, current_unix_secs()?);
            let report = build_icrc_account_info_report(&request).await?;
            write_text_or_json(format, &report, icrc_account_info_report_text)
        }
        Some(("list", matches)) => {
            let (format, request) = analytics_account_list_request(matches, current_unix_secs()?);
            let report = build_icrc_account_list_report(&request).await?;
            write_text_or_json(format, &report, icrc_account_list_report_text)
        }
        _ => unreachable!("clap requires a known ICRC account analytics subcommand"),
    }
}

async fn run_icrc_analytics_holder(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("count", matches)) => {
            run_icrc_analytics_indexed_count_leaf(matches, IcIcrcIndexedCountKind::Holder).await
        }
        Some(("list", matches)) => {
            let (format, request) = analytics_holder_list_request(matches, current_unix_secs()?);
            let report = build_icrc_holder_list_report(&request).await?;
            write_text_or_json(format, &report, icrc_holder_list_report_text)
        }
        _ => unreachable!("clap requires a known ICRC holder analytics subcommand"),
    }
}

async fn run_icrc_analytics_token_values(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = analytics_token_value_request(matches, current_unix_secs()?);
    let report = build_icrc_token_value_report(&request).await?;
    write_text_or_json(format, &report, icrc_token_value_report_text)
}

async fn run_icrc_analytics_indexed_count_leaf(
    matches: &ArgMatches,
    kind: IcIcrcIndexedCountKind,
) -> Result<(), IcrcCommandError> {
    let options = IcrcLedgerOptions::from_matches(matches);
    let (format, dashboard, ledger_canister_id) =
        options.into_dashboard_request(current_unix_secs()?);
    let request = IcIcrcIndexedCountRequest::new(dashboard, ledger_canister_id, kind);
    let report = build_icrc_indexed_count_report(&request).await?;
    write_text_or_json(format, &report, icrc_indexed_count_report_text)
}

async fn run_icrc_analytics_total_supply(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = analytics_total_supply_request(matches, current_unix_secs()?);
    let report = build_icrc_total_supply_report(&request).await?;
    write_text_or_json(format, &report, icrc_total_supply_report_text)
}

async fn run_icrc_ledger(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("token", matches)) => {
            run_simple_ledger_report(matches, build_icrc_token_report, icrc_token_report_text).await
        }
        Some(("index", matches)) => {
            run_simple_ledger_report(matches, build_icrc_index_report, icrc_index_report_text).await
        }
        Some(("transactions", matches)) => run_icrc_transactions(matches).await,
        Some(("block-types", matches)) => {
            run_simple_ledger_report(
                matches,
                build_icrc_block_types_report,
                icrc_block_types_report_text,
            )
            .await
        }
        Some(("archives", matches)) => run_icrc_archives(matches).await,
        Some(("tip-certificate", matches)) => {
            run_simple_ledger_report(
                matches,
                build_icrc_tip_certificate_report,
                icrc_tip_certificate_report_text,
            )
            .await
        }
        Some(("capabilities", matches)) => {
            run_simple_ledger_report(
                matches,
                build_icrc_capabilities_report,
                icrc_capabilities_report_text,
            )
            .await
        }
        _ => unreachable!("clap requires a known ICRC ledger subcommand"),
    }
}

async fn run_simple_ledger_report<Report>(
    matches: &ArgMatches,
    build: impl AsyncFn(&IcrcLedgerRequest) -> Result<Report, IcrcError>,
    render_text: fn(&Report) -> String,
) -> Result<(), IcrcCommandError>
where
    Report: Serialize,
{
    let options = IcrcLedgerOptions::from_matches(matches);
    let (format, request) = options.into_ledger_request(current_unix_secs()?);
    let report = build(&request).await?;
    write_text_or_json(format, &report, render_text)
}

async fn run_icrc_account(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("balance", matches)) => run_icrc_balance(matches).await,
        Some(("allowance", matches)) => run_icrc_allowance(matches).await,
        Some(("transaction", matches)) => run_icrc_account_transaction(matches).await,
        _ => unreachable!("clap requires a known ICRC account subcommand"),
    }
}

async fn run_icrc_balance(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let options = IcrcAccountOptions::from_matches(matches);
    let (format, ledger) = options.ledger.into_ledger_request(current_unix_secs()?);
    let request = IcrcBalanceRequest {
        ledger,
        account_owner: options.account_owner,
        subaccount_hex: options.subaccount_hex,
    };
    let report = build_icrc_balance_report(&request).await?;
    write_text_or_json(format, &report, icrc_balance_report_text)
}

async fn run_icrc_allowance(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = allowance_request(matches, current_unix_secs()?);
    let report = build_icrc_allowance_report(&request).await?;
    write_text_or_json(format, &report, icrc_allowance_report_text)
}

async fn run_icrc_account_transaction(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    match matches.subcommand() {
        Some(("page", matches)) => run_icrc_account_transaction_page(matches).await,
        Some(("list", matches)) => run_icrc_account_transaction_list(matches),
        Some(("refresh", matches)) => run_icrc_account_transaction_refresh(matches).await,
        Some(("cache", matches)) => run_icrc_account_transaction_cache(matches),
        _ => unreachable!("clap requires a known ICRC account transaction subcommand"),
    }
}

async fn run_icrc_account_transaction_page(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = account_transaction_page_request(matches, current_unix_secs()?);
    let report = build_icrc_account_transaction_page_report(&request).await?;
    write_text_or_json(format, &report, icrc_account_transaction_page_report_text)
}

fn run_icrc_account_transaction_list(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) =
        account_transaction_list_request(matches, account_transaction_cache_root()?);
    let report = build_icrc_account_transaction_list_report(&request)?;
    write_text_or_json(format, &report, icrc_account_transaction_list_report_text)
}

async fn run_icrc_account_transaction_refresh(
    matches: &ArgMatches,
) -> Result<(), IcrcCommandError> {
    let (format, request) = account_transaction_refresh_request(
        matches,
        account_transaction_cache_root()?,
        current_unix_secs()?,
    );
    let mut progress = StderrQueryProgress::new();
    let report =
        refresh_icrc_account_transaction_cache_with_progress(&request, &mut progress).await?;
    write_text_or_json(
        format,
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
    let options = IcrcAccountOptions::from_matches(matches);
    let (format, request) = options.into_cache_request(account_transaction_cache_root()?);
    let report = build_icrc_account_transaction_cache_status_report(&request)?;
    write_text_or_json(
        format,
        &report,
        icrc_account_transaction_cache_status_report_text,
    )
}

fn account_transaction_cache_root() -> Result<std::path::PathBuf, IcrcCommandError> {
    cache_root().map_err(|error| IcrcCommandError::Usage(error.to_string()))
}

async fn run_icrc_transactions(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = transactions_request(matches, current_unix_secs()?);
    let report = build_icrc_transactions_report(&request).await?;
    write_text_or_json(format, &report, icrc_transactions_report_text)
}

async fn run_icrc_archives(matches: &ArgMatches) -> Result<(), IcrcCommandError> {
    let (format, request) = archives_request(matches, current_unix_secs()?);
    let report = build_icrc_archives_report(&request).await?;
    write_text_or_json(format, &report, icrc_archives_report_text)
}
