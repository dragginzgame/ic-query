//! Module: icrc::commands
//!
//! Responsibility: parse and run generic ICRC CLI commands.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: maps clap options into report requests and writes text/JSON output.

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help, required_string, required_typed, string_option, value_arg,
        },
        common::{
            OutputFormat, current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        help::collect_args_or_print_help_or_version,
    },
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    icrc::{
        live::{
            build_icrc_allowance_report, build_icrc_balance_report, build_icrc_index_report,
            build_icrc_token_report, build_icrc_transactions_report,
        },
        model::{
            IcrcAllowanceRequest, IcrcBalanceRequest, IcrcError, IcrcIndexRequest,
            IcrcTokenRequest, IcrcTransactionsRequest, normalize_subaccount_hex,
        },
        text::{
            icrc_allowance_report_text, icrc_balance_report_text, icrc_index_report_text,
            icrc_token_report_text, icrc_transactions_report_text,
        },
    },
    version_text,
};
use candid::Principal;
use clap::{
    ArgMatches, Command as ClapCommand,
    builder::{RangedU64ValueParser, ValueParser},
};
use std::ffi::OsString;

pub(in crate::icrc) const DEFAULT_ICRC_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
const DEFAULT_ICRC_TRANSACTIONS_LIMIT: &str = "25";
const MAX_ICRC_TRANSACTIONS_LIMIT: u64 = 100;
const LEDGER_CANISTER_ID_ARG: &str = "ledger-canister-id";
const PRINCIPAL_ARG: &str = "principal";
const OWNER_PRINCIPAL_ARG: &str = "owner-principal";
const SPENDER_PRINCIPAL_ARG: &str = "spender-principal";
const SUBACCOUNT_ARG: &str = "subaccount";
const OWNER_SUBACCOUNT_ARG: &str = "owner-subaccount";
const SPENDER_SUBACCOUNT_ARG: &str = "spender-subaccount";
const START_ARG: &str = "start";
const LIMIT_ARG: &str = "limit";
const FORMAT_ARG: &str = "format";
const SOURCE_ENDPOINT_ARG: &str = "source-endpoint";

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
        "allowance" => run_icrc_allowance(args),
        "index" => run_icrc_index(args),
        "transactions" => run_icrc_transactions(args),
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

fn run_icrc_allowance<I>(args: I) -> Result<(), IcrcError>
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

fn run_icrc_index<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help_or_version(args, icrc_index_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcIndexOptions::parse(args)?;
    let request = IcrcIndexRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_index_report(&request)?;
    write_text_or_json(options.format, &report, icrc_index_report_text)
}

fn run_icrc_transactions<I>(args: I) -> Result<(), IcrcError>
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
    };
    let report = build_icrc_transactions_report(&request)?;
    write_text_or_json(options.format, &report, icrc_transactions_report_text)
}

fn icrc_command() -> ClapCommand {
    ClapCommand::new("icrc")
        .bin_name("icq icrc")
        .about("Inspect generic ICRC ledgers")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("token")
                .about("Show generic ICRC token metadata by ledger canister id"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("balance").about("Show a generic ICRC account balance"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("allowance").about("Show a generic ICRC account allowance"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("index").about("Show a generic ICRC ledger index canister"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("transactions")
                .about("Show a generic ICRC ledger transaction history page"),
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
        .arg(ledger_canister_id_arg())
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
        .arg(ledger_canister_id_arg())
        .arg(principal_arg(PRINCIPAL_ARG, "Account owner principal"))
        .arg(subaccount_arg(
            SUBACCOUNT_ARG,
            "Optional 32-byte ICRC subaccount as hex",
        ))
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for ICRC ledger queries"),
        )
        .arg(format_arg())
}

fn icrc_allowance_command() -> ClapCommand {
    ClapCommand::new("allowance")
        .bin_name("icq icrc allowance")
        .about("Show a generic ICRC account allowance")
        .after_help(
            "Examples:\n  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa\n  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        )
        .disable_help_flag(true)
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
        ))
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for ICRC ledger queries"),
        )
        .arg(format_arg())
}

fn icrc_index_command() -> ClapCommand {
    ClapCommand::new("index")
        .bin_name("icq icrc index")
        .about("Show a generic ICRC ledger index canister")
        .after_help(
            "Examples:\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg())
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for ICRC ledger queries"),
        )
        .arg(format_arg())
}

fn icrc_transactions_command() -> ClapCommand {
    ClapCommand::new("transactions")
        .bin_name("icq icrc transactions")
        .about("Show a generic ICRC ledger transaction history page")
        .after_help(
            "Examples:\n  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai --start 100 --limit 50 --format json",
        )
        .disable_help_flag(true)
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

fn icrc_allowance_usage() -> String {
    render_help(icrc_allowance_command())
}

fn icrc_index_usage() -> String {
    render_help(icrc_index_command())
}

fn icrc_transactions_usage() -> String {
    render_help(icrc_transactions_command())
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
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
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
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            account_owner: required_string(&matches, PRINCIPAL_ARG),
            subaccount_hex: string_option(&matches, SUBACCOUNT_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcAllowanceOptions
///
/// Clap-parsed options for generic ICRC allowance queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAllowanceOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) account_owner: String,
    pub(in crate::icrc) account_subaccount_hex: Option<String>,
    pub(in crate::icrc) spender_owner: String,
    pub(in crate::icrc) spender_subaccount_hex: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcAllowanceOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_allowance_command(), args, icrc_allowance_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            account_owner: required_string(&matches, OWNER_PRINCIPAL_ARG),
            account_subaccount_hex: string_option(&matches, OWNER_SUBACCOUNT_ARG),
            spender_owner: required_string(&matches, SPENDER_PRINCIPAL_ARG),
            spender_subaccount_hex: string_option(&matches, SPENDER_SUBACCOUNT_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcIndexOptions
///
/// Clap-parsed options for generic ICRC index discovery queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcIndexOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcIndexOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_index_command(), args, icrc_index_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcTransactionsOptions
///
/// Clap-parsed options for generic ICRC transaction history queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTransactionsOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) start: u64,
    pub(in crate::icrc) limit: u32,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTransactionsOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_transactions_command(), args, icrc_transactions_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            start: required_typed(&matches, START_ARG),
            limit: required_typed(&matches, LIMIT_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

fn ledger_canister_id_arg() -> clap::Arg {
    principal_arg(LEDGER_CANISTER_ID_ARG, "ICRC ledger canister principal")
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
    *matches
        .get_one::<OutputFormat>(FORMAT_ARG)
        .expect("clap requires format default")
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

#[cfg(test)]
pub(in crate::icrc) mod test_support {
    use super::{
        IcrcAllowanceOptions, IcrcBalanceOptions, IcrcIndexOptions, IcrcTokenOptions,
        IcrcTransactionsOptions, icrc_allowance_usage, icrc_balance_usage, icrc_index_usage,
        icrc_token_usage, icrc_transactions_usage, usage,
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

    pub(in crate::icrc) fn parse_allowance_options(args: &[&str]) -> IcrcAllowanceOptions {
        try_parse_allowance_options(args).expect("parse ICRC allowance options")
    }

    pub(in crate::icrc) fn try_parse_allowance_options(
        args: &[&str],
    ) -> Result<IcrcAllowanceOptions, crate::icrc::model::IcrcError> {
        IcrcAllowanceOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_index_options(args: &[&str]) -> IcrcIndexOptions {
        try_parse_index_options(args).expect("parse ICRC index options")
    }

    pub(in crate::icrc) fn try_parse_index_options(
        args: &[&str],
    ) -> Result<IcrcIndexOptions, crate::icrc::model::IcrcError> {
        IcrcIndexOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_transactions_options(args: &[&str]) -> IcrcTransactionsOptions {
        try_parse_transactions_options(args).expect("parse ICRC transactions options")
    }

    pub(in crate::icrc) fn try_parse_transactions_options(
        args: &[&str],
    ) -> Result<IcrcTransactionsOptions, crate::icrc::model::IcrcError> {
        IcrcTransactionsOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
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

    pub(in crate::icrc) fn allowance_usage() -> String {
        icrc_allowance_usage()
    }

    pub(in crate::icrc) fn index_usage() -> String {
        icrc_index_usage()
    }

    pub(in crate::icrc) fn transactions_usage() -> String {
        icrc_transactions_usage()
    }
}
