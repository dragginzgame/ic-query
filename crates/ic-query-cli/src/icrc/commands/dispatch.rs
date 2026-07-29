//! Module: icrc::commands::dispatch
//!
//! Responsibility: dispatch parsed ICRC commands into report builders and output.
//! Does not own: Clap command definitions, option parsing, live calls, or rendering.
//! Boundary: converts command options to public requests and writes one report.

use super::{
    IcrcAllowanceOptions, IcrcArchivesOptions, IcrcBalanceOptions, IcrcLedgerOptions,
    IcrcTransactionsOptions, icrc_allowance_usage, icrc_archives_usage, icrc_balance_usage,
    icrc_block_types_command, icrc_block_types_usage, icrc_capabilities_command,
    icrc_capabilities_usage, icrc_command, icrc_index_command, icrc_index_usage,
    icrc_tip_certificate_command, icrc_tip_certificate_usage, icrc_token_command, icrc_token_usage,
    icrc_transactions_usage, usage,
};
use crate::{
    cli::{
        clap::parse_required_subcommand_or_usage,
        common::{current_unix_secs, write_text_or_json},
        help::collect_args_or_print_help_or_version,
    },
    icrc::IcrcCommandError,
    version_text,
};
use ic_query::icrc::{
    IcrcAllowanceRequest, IcrcArchivesRequest, IcrcBalanceRequest, IcrcBlockTypesRequest,
    IcrcCapabilitiesRequest, IcrcIndexRequest, IcrcTipCertificateRequest, IcrcTokenRequest,
    IcrcTransactionsRequest, build_icrc_allowance_report, build_icrc_archives_report,
    build_icrc_balance_report, build_icrc_block_types_report, build_icrc_capabilities_report,
    build_icrc_index_report, build_icrc_tip_certificate_report, build_icrc_token_report,
    build_icrc_transactions_report, icrc_allowance_report_text, icrc_archives_report_text,
    icrc_balance_report_text, icrc_block_types_report_text, icrc_capabilities_report_text,
    icrc_index_report_text, icrc_tip_certificate_report_text, icrc_token_report_text,
    icrc_transactions_report_text,
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
        "token" => run_icrc_token(args),
        "balance" => run_icrc_balance(args),
        "allowance" => run_icrc_allowance(args),
        "index" => run_icrc_index(args),
        "transactions" => run_icrc_transactions(args),
        "block-types" => run_icrc_block_types(args),
        "archives" => run_icrc_archives(args),
        "tip-certificate" => run_icrc_tip_certificate(args),
        "capabilities" => run_icrc_capabilities(args),
        _ => unreachable!("ICRC command only defines known subcommands"),
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
