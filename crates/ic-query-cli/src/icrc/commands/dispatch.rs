//! Module: icrc::commands::dispatch
//!
//! Responsibility: dispatch parsed ICRC commands into report builders and output.
//! Does not own: Clap command definitions, option parsing, live calls, or rendering.
//! Boundary: converts command options to public requests and writes one report.

use super::{
    IcrcAccountTransactionCacheOptions, IcrcAccountTransactionListOptions,
    IcrcAccountTransactionPageOptions, IcrcAccountTransactionRefreshOptions, IcrcAllowanceOptions,
    IcrcArchivesOptions, IcrcBalanceOptions, IcrcLedgerOptions, IcrcTransactionsOptions,
    icrc_account_command, icrc_account_transaction_cache_command,
    icrc_account_transaction_cache_status_usage, icrc_account_transaction_cache_usage,
    icrc_account_transaction_command, icrc_account_transaction_list_usage,
    icrc_account_transaction_page_usage, icrc_account_transaction_refresh_usage,
    icrc_account_transaction_usage, icrc_account_usage, icrc_allowance_usage, icrc_archives_usage,
    icrc_balance_usage, icrc_block_types_command, icrc_block_types_usage,
    icrc_capabilities_command, icrc_capabilities_usage, icrc_command, icrc_index_command,
    icrc_index_usage, icrc_ledger_command, icrc_ledger_usage, icrc_tip_certificate_command,
    icrc_tip_certificate_usage, icrc_token_command, icrc_token_usage, icrc_transactions_usage,
    usage,
};
use crate::{
    cli::{
        clap::parse_required_subcommand_or_usage,
        common::{current_unix_secs, write_text_or_json},
        help::collect_args_or_print_help_or_version,
    },
    icrc::IcrcCommandError,
    progress::StderrQueryProgress,
    project::icp_root,
    version_text,
};
use ic_query::icrc::{
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    IcrcAccountTransactionCacheRequest, IcrcAccountTransactionListRequest,
    IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshRequest, IcrcAllowanceRequest,
    IcrcArchivesRequest, IcrcBalanceRequest, IcrcBlockTypesRequest, IcrcCapabilitiesRequest,
    IcrcIndexRequest, IcrcTipCertificateRequest, IcrcTokenRequest, IcrcTransactionsRequest,
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
use std::ffi::OsString;

pub fn run<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, usage, version_text()) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(icrc_command(), args, usage)
        .map_err(IcrcCommandError::Usage)?;
    match command.as_str() {
        "ledger" => run_icrc_ledger(args),
        "account" => run_icrc_account(args),
        _ => unreachable!("ICRC command only defines known subcommands"),
    }
}

fn run_icrc_ledger<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, icrc_ledger_usage, version_text())
    else {
        return Ok(());
    };
    let (command, args) =
        parse_required_subcommand_or_usage(icrc_ledger_command(), args, icrc_ledger_usage)
            .map_err(IcrcCommandError::Usage)?;
    match command.as_str() {
        "token" => run_icrc_token(args),
        "index" => run_icrc_index(args),
        "transactions" => run_icrc_transactions(args),
        "block-types" => run_icrc_block_types(args),
        "archives" => run_icrc_archives(args),
        "tip-certificate" => run_icrc_tip_certificate(args),
        "capabilities" => run_icrc_capabilities(args),
        _ => unreachable!("ICRC ledger command only defines known subcommands"),
    }
}

fn run_icrc_account<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_account_usage, version_text())
    else {
        return Ok(());
    };
    let (command, args) =
        parse_required_subcommand_or_usage(icrc_account_command(), args, icrc_account_usage)
            .map_err(IcrcCommandError::Usage)?;
    match command.as_str() {
        "balance" => run_icrc_balance(args),
        "allowance" => run_icrc_allowance(args),
        "transaction" => run_icrc_account_transaction(args),
        _ => unreachable!("ICRC account command only defines known subcommands"),
    }
}

fn run_icrc_token<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, icrc_token_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcLedgerOptions::parse(args, icrc_token_command, icrc_token_usage)?;
    let request = IcrcTokenRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_token_report(&request)?;
    write_text_or_json(options.format, &report, icrc_token_report_text)
}

fn run_icrc_balance<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_balance_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcBalanceOptions::parse(args)?;
    let request = IcrcBalanceRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
        account_owner: options.account_owner,
        subaccount_hex: options.subaccount_hex,
    };
    let report = build_icrc_balance_report(&request)?;
    write_text_or_json(options.format, &report, icrc_balance_report_text)
}

fn run_icrc_allowance<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_allowance_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcAllowanceOptions::parse(args)?;
    let request = IcrcAllowanceRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
        account_owner: options.account_owner,
        account_subaccount_hex: options.account_subaccount_hex,
        spender_owner: options.spender_owner,
        spender_subaccount_hex: options.spender_subaccount_hex,
    };
    let report = build_icrc_allowance_report(&request)?;
    write_text_or_json(options.format, &report, icrc_allowance_report_text)
}

fn run_icrc_account_transaction<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_account_transaction_usage, version_text())
    else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(
        icrc_account_transaction_command(),
        args,
        icrc_account_transaction_usage,
    )
    .map_err(IcrcCommandError::Usage)?;
    match command.as_str() {
        "page" => run_icrc_account_transaction_page(args),
        "list" => run_icrc_account_transaction_list(args),
        "refresh" => run_icrc_account_transaction_refresh(args),
        "cache" => run_icrc_account_transaction_cache(args),
        _ => unreachable!("ICRC account transaction command only defines known subcommands"),
    }
}

fn run_icrc_account_transaction_page<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(
        args,
        icrc_account_transaction_page_usage,
        version_text(),
    ) else {
        return Ok(());
    };
    let options = IcrcAccountTransactionPageOptions::parse(args)?;
    let request = IcrcAccountTransactionPageRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
        index_canister_id: options.index_canister_id,
        account_owner: options.account_owner,
        subaccount_hex: options.subaccount_hex,
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

fn run_icrc_account_transaction_list<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(
        args,
        icrc_account_transaction_list_usage,
        version_text(),
    ) else {
        return Ok(());
    };
    let options = IcrcAccountTransactionListOptions::parse(args)?;
    let request = IcrcAccountTransactionListRequest {
        cache: account_transaction_cache_request(
            options.source_endpoint,
            options.ledger_canister_id,
            options.account_owner,
            options.subaccount_hex,
        )?,
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

fn run_icrc_account_transaction_refresh<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(
        args,
        icrc_account_transaction_refresh_usage,
        version_text(),
    ) else {
        return Ok(());
    };
    let options = IcrcAccountTransactionRefreshOptions::parse(args)?;
    let request = IcrcAccountTransactionRefreshRequest {
        cache: account_transaction_cache_request(
            options.source_endpoint,
            options.ledger_canister_id,
            options.account_owner,
            options.subaccount_hex,
        )?,
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

fn run_icrc_account_transaction_cache<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(
        args,
        icrc_account_transaction_cache_usage,
        version_text(),
    ) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(
        icrc_account_transaction_cache_command(),
        args,
        icrc_account_transaction_cache_usage,
    )
    .map_err(IcrcCommandError::Usage)?;
    match command.as_str() {
        "status" => run_icrc_account_transaction_cache_status(args),
        _ => unreachable!("ICRC account transaction cache command only defines known subcommands"),
    }
}

fn run_icrc_account_transaction_cache_status<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(
        args,
        icrc_account_transaction_cache_status_usage,
        version_text(),
    ) else {
        return Ok(());
    };
    let options = IcrcAccountTransactionCacheOptions::parse(args)?;
    let request = account_transaction_cache_request(
        options.source_endpoint,
        options.ledger_canister_id,
        options.account_owner,
        options.subaccount_hex,
    )?;
    let report = build_icrc_account_transaction_cache_status_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        icrc_account_transaction_cache_status_report_text,
    )
}

fn account_transaction_cache_request(
    source_endpoint: String,
    ledger_canister_id: String,
    account_owner: String,
    subaccount_hex: Option<String>,
) -> Result<IcrcAccountTransactionCacheRequest, IcrcCommandError> {
    Ok(IcrcAccountTransactionCacheRequest {
        icp_root: icp_root().map_err(|error| IcrcCommandError::Usage(error.to_string()))?,
        source_endpoint,
        ledger_canister_id,
        account_owner,
        subaccount_hex,
    })
}

fn run_icrc_index<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, icrc_index_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcLedgerOptions::parse(args, icrc_index_command, icrc_index_usage)?;
    let request = IcrcIndexRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_index_report(&request)?;
    write_text_or_json(options.format, &report, icrc_index_report_text)
}

fn run_icrc_transactions<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_transactions_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcTransactionsOptions::parse(args)?;
    let request = IcrcTransactionsRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
        start: options.start,
        limit: options.limit,
        follow_archives: options.follow_archives,
    };
    let report = build_icrc_transactions_report(&request)?;
    write_text_or_json(options.format, &report, icrc_transactions_report_text)
}

fn run_icrc_block_types<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_block_types_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcLedgerOptions::parse(args, icrc_block_types_command, icrc_block_types_usage)?;
    let request = IcrcBlockTypesRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_block_types_report(&request)?;
    write_text_or_json(options.format, &report, icrc_block_types_report_text)
}

fn run_icrc_archives<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_archives_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcArchivesOptions::parse(args)?;
    let request = IcrcArchivesRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
        from_canister_id: options.from_canister_id,
    };
    let report = build_icrc_archives_report(&request)?;
    write_text_or_json(options.format, &report, icrc_archives_report_text)
}

fn run_icrc_tip_certificate<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_tip_certificate_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcLedgerOptions::parse(
        args,
        icrc_tip_certificate_command,
        icrc_tip_certificate_usage,
    )?;
    let request = IcrcTipCertificateRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_tip_certificate_report(&request)?;
    write_text_or_json(options.format, &report, icrc_tip_certificate_report_text)
}

fn run_icrc_capabilities<I>(args: I) -> Result<(), IcrcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_capabilities_usage, version_text())
    else {
        return Ok(());
    };
    let options =
        IcrcLedgerOptions::parse(args, icrc_capabilities_command, icrc_capabilities_usage)?;
    let request = IcrcCapabilitiesRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_capabilities_report(&request)?;
    write_text_or_json(options.format, &report, icrc_capabilities_report_text)
}
