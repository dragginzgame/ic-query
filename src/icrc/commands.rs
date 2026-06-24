//! Module: icrc::commands
//!
//! Responsibility: parse and run generic ICRC CLI commands.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: maps clap options into report requests and writes text/JSON output.

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help, required_string, string_option, value_arg,
        },
        common::{
            OutputFormat, current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        help::collect_args_or_print_help_or_version,
    },
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    icrc::{
        live::{build_icrc_balance_report, build_icrc_token_report},
        model::{IcrcBalanceRequest, IcrcError, IcrcTokenRequest, normalize_subaccount_hex},
        text::{icrc_balance_report_text, icrc_token_report_text},
    },
    version_text,
};
use candid::Principal;
use clap::{Command as ClapCommand, builder::ValueParser};
use std::ffi::OsString;

pub(in crate::icrc) const DEFAULT_ICRC_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;

pub fn run<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, usage, version_text()) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(icrc_command(), args, usage)
        .map_err(IcrcError::Usage)?;
    match command.as_str() {
        "token" => run_icrc_token(args),
        "balance" => run_icrc_balance(args),
        _ => unreachable!("ICRC command only defines known subcommands"),
    }
}

fn run_icrc_token<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, icrc_token_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcTokenOptions::parse(args)?;
    let request = IcrcTokenRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_token_report(&request)?;
    write_text_or_json(options.format, &report, icrc_token_report_text)
}

fn run_icrc_balance<I>(args: I) -> Result<(), IcrcError>
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

fn icrc_command() -> ClapCommand {
    ClapCommand::new("icrc")
        .bin_name("icq icrc")
        .about("Inspect generic ICRC ledger metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("token")
                .about("Show generic ICRC token metadata by ledger canister id"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("balance").about("Show a generic ICRC account balance"),
        ))
}

fn icrc_token_command() -> ClapCommand {
    ClapCommand::new("token")
        .bin_name("icq icrc token")
        .about("Show generic ICRC token metadata by ledger canister id")
        .after_help(
            "Examples:\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        )
        .disable_help_flag(true)
        .arg(
            value_arg("ledger-canister-id")
                .value_name("ledger-canister-id")
                .required(true)
                .value_parser(principal_text_value_parser())
                .help("ICRC ledger canister principal"),
        )
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for ICRC ledger queries"),
        )
        .arg(format_arg())
}

fn icrc_balance_command() -> ClapCommand {
    ClapCommand::new("balance")
        .bin_name("icq icrc balance")
        .about("Show a generic ICRC account balance")
        .after_help(
            "Examples:\n  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa\n  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        )
        .disable_help_flag(true)
        .arg(
            value_arg("ledger-canister-id")
                .value_name("ledger-canister-id")
                .required(true)
                .value_parser(principal_text_value_parser())
                .help("ICRC ledger canister principal"),
        )
        .arg(
            value_arg("principal")
                .value_name("principal")
                .required(true)
                .value_parser(principal_text_value_parser())
                .help("Account owner principal"),
        )
        .arg(
            value_arg("subaccount")
                .long("subaccount")
                .value_name("hex")
                .value_parser(subaccount_hex_value_parser())
                .help("Optional 32-byte ICRC subaccount as hex"),
        )
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for ICRC ledger queries"),
        )
        .arg(format_arg())
}

fn usage() -> String {
    render_help(icrc_command())
}

fn icrc_token_usage() -> String {
    render_help(icrc_token_command())
}

fn icrc_balance_usage() -> String {
    render_help(icrc_balance_command())
}

///
/// IcrcTokenOptions
///
/// Clap-parsed options for generic ICRC token metadata queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTokenOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTokenOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_token_command(), args, icrc_token_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, "ledger-canister-id"),
            format: *matches
                .get_one::<OutputFormat>("format")
                .expect("clap requires format default"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        })
    }
}

///
/// IcrcBalanceOptions
///
/// Clap-parsed options for generic ICRC account balance queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBalanceOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) subaccount_hex: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcBalanceOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_balance_command(), args, icrc_balance_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, "ledger-canister-id"),
            account_owner: required_string(&matches, "principal"),
            subaccount_hex: string_option(&matches, "subaccount"),
            format: *matches
                .get_one::<OutputFormat>("format")
                .expect("clap requires format default"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        })
    }
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

#[cfg(test)]
pub(in crate::icrc) mod test_support {
    use super::{
        IcrcBalanceOptions, IcrcTokenOptions, icrc_balance_usage, icrc_token_usage, usage,
    };

    pub(in crate::icrc) fn parse_token_options(args: &[&str]) -> IcrcTokenOptions {
        try_parse_token_options(args).expect("parse ICRC token options")
    }

    pub(in crate::icrc) fn try_parse_token_options(
        args: &[&str],
    ) -> Result<IcrcTokenOptions, crate::icrc::model::IcrcError> {
        IcrcTokenOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_balance_options(args: &[&str]) -> IcrcBalanceOptions {
        try_parse_balance_options(args).expect("parse ICRC balance options")
    }

    pub(in crate::icrc) fn try_parse_balance_options(
        args: &[&str],
    ) -> Result<IcrcBalanceOptions, crate::icrc::model::IcrcError> {
        IcrcBalanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn root_usage() -> String {
        usage()
    }

    pub(in crate::icrc) fn token_usage() -> String {
        icrc_token_usage()
    }

    pub(in crate::icrc) fn balance_usage() -> String {
        icrc_balance_usage()
    }
}
