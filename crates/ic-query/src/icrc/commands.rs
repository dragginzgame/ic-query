//! Module: icrc::commands
//!
//! Responsibility: parse and run generic ICRC CLI commands.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: maps clap options into report requests and writes text/JSON output.

use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches_or_usage, parse_required_subcommand_or_usage,
            passthrough_subcommand, render_help, required_string, required_typed, string_option,
            value_arg,
        },
        common::{
            OutputFormat, current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        help::collect_args_or_print_help_or_version,
    },
    icrc::{
        DEFAULT_ICRC_SOURCE_ENDPOINT,
        live::{
            build_icrc_allowance_report, build_icrc_archives_report, build_icrc_balance_report,
            build_icrc_block_types_report, build_icrc_capabilities_report, build_icrc_index_report,
            build_icrc_tip_certificate_report, build_icrc_token_report,
            build_icrc_transactions_report,
        },
        model::{
            IcrcAllowanceRequest, IcrcArchivesRequest, IcrcBalanceRequest, IcrcBlockTypesRequest,
            IcrcCapabilitiesRequest, IcrcError, IcrcIndexRequest, IcrcTipCertificateRequest,
            IcrcTokenRequest, IcrcTransactionsRequest, normalize_subaccount_hex,
        },
        text::{
            icrc_allowance_report_text, icrc_archives_report_text, icrc_balance_report_text,
            icrc_block_types_report_text, icrc_capabilities_report_text, icrc_index_report_text,
            icrc_tip_certificate_report_text, icrc_token_report_text,
            icrc_transactions_report_text,
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
const FOLLOW_ARCHIVES_ARG: &str = "follow-archives";
const FROM_CANISTER_ID_ARG: &str = "from";
const FORMAT_ARG: &str = "format";
const SOURCE_ENDPOINT_ARG: &str = "source-endpoint";
const ICRC_SOURCE_ENDPOINT_HELP: &str = "IC API endpoint used for ICRC ledger queries";

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
        "block-types" => run_icrc_block_types(args),
        "archives" => run_icrc_archives(args),
        "tip-certificate" => run_icrc_tip_certificate(args),
        "capabilities" => run_icrc_capabilities(args),
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
        follow_archives: options.follow_archives,
    };
    let report = build_icrc_transactions_report(&request)?;
    write_text_or_json(options.format, &report, icrc_transactions_report_text)
}

fn run_icrc_block_types<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_block_types_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcBlockTypesOptions::parse(args)?;
    let request = IcrcBlockTypesRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_block_types_report(&request)?;
    write_text_or_json(options.format, &report, icrc_block_types_report_text)
}

fn run_icrc_archives<I>(args: I) -> Result<(), IcrcError>
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

fn run_icrc_tip_certificate<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_tip_certificate_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcTipCertificateOptions::parse(args)?;
    let request = IcrcTipCertificateRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_tip_certificate_report(&request)?;
    write_text_or_json(options.format, &report, icrc_tip_certificate_report_text)
}

fn run_icrc_capabilities<I>(args: I) -> Result<(), IcrcError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) =
        collect_args_or_print_help_or_version(args, icrc_capabilities_usage, version_text())
    else {
        return Ok(());
    };
    let options = IcrcCapabilitiesOptions::parse(args)?;
    let request = IcrcCapabilitiesRequest {
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs()?,
        ledger_canister_id: options.ledger_canister_id,
    };
    let report = build_icrc_capabilities_report(&request)?;
    write_text_or_json(options.format, &report, icrc_capabilities_report_text)
}

fn icrc_command() -> ClapCommand {
    ClapCommand::new("icrc")
        .bin_name("icq icrc")
        .about("Inspect generic ICRC ledgers")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("capabilities")
                .about("Probe generic ICRC ledger endpoint capabilities"),
        ))
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
        .subcommand(passthrough_subcommand(
            ClapCommand::new("block-types")
                .about("Show generic ICRC-3 ledger supported block types"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("archives").about("Show generic ICRC-3 ledger archive ranges"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("tip-certificate")
                .about("Show a generic ICRC-3 ledger tip certificate"),
        ))
}

fn icrc_token_command() -> ClapCommand {
    let command = ClapCommand::new("token")
        .bin_name("icq icrc token")
        .about("Show generic ICRC token metadata by ledger canister id")
        .after_help(
            "Examples:\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_capabilities_command() -> ClapCommand {
    let command = ClapCommand::new("capabilities")
        .bin_name("icq icrc capabilities")
        .about("Probe generic ICRC ledger endpoint capabilities")
        .after_help(
            "Examples:\n  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_balance_command() -> ClapCommand {
    let command = ClapCommand::new("balance")
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
        ));
    with_common_icrc_options(command)
}

fn icrc_allowance_command() -> ClapCommand {
    let command = ClapCommand::new("allowance")
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
        ));
    with_common_icrc_options(command)
}

fn icrc_index_command() -> ClapCommand {
    let command = ClapCommand::new("index")
        .bin_name("icq icrc index")
        .about("Show a generic ICRC ledger index canister")
        .after_help(
            "Examples:\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_transactions_command() -> ClapCommand {
    let command = ClapCommand::new("transactions")
        .bin_name("icq icrc transactions")
        .about("Show a generic ICRC ledger transaction history page")
        .after_help(
            "Examples:\n  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --format json",
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
        );
    let command = with_icrc_source_endpoint_option(command).arg(
        flag_arg(FOLLOW_ARCHIVES_ARG)
            .long(FOLLOW_ARCHIVES_ARG)
            .help("Follow returned ICRC-3 archive callbacks for the requested block page"),
    );
    with_icrc_format_option(command)
}

fn icrc_block_types_command() -> ClapCommand {
    let command = ClapCommand::new("block-types")
        .bin_name("icq icrc block-types")
        .about("Show generic ICRC-3 ledger supported block types")
        .after_help(
            "Examples:\n  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_archives_command() -> ClapCommand {
    let command = ClapCommand::new("archives")
        .bin_name("icq icrc archives")
        .about("Show generic ICRC-3 ledger archive ranges")
        .after_help(
            "Examples:\n  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --format json",
        )
        .disable_help_flag(true)
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
    let command = ClapCommand::new("tip-certificate")
        .bin_name("icq icrc tip-certificate")
        .about("Show a generic ICRC-3 ledger tip certificate")
        .after_help(
            "Examples:\n  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --format json",
        )
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn usage() -> String {
    render_help(icrc_command())
}

fn icrc_token_usage() -> String {
    render_help(icrc_token_command())
}

fn icrc_capabilities_usage() -> String {
    render_help(icrc_capabilities_command())
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

fn icrc_block_types_usage() -> String {
    render_help(icrc_block_types_command())
}

fn icrc_archives_usage() -> String {
    render_help(icrc_archives_command())
}

fn icrc_tip_certificate_usage() -> String {
    render_help(icrc_tip_certificate_command())
}

///
/// IcrcCapabilitiesOptions
///
/// Clap-parsed options for generic ICRC capability probes.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcCapabilitiesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcCapabilitiesOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_capabilities_command(), args, icrc_capabilities_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
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
    pub(in crate::icrc) follow_archives: bool,
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
            follow_archives: matches.get_flag(FOLLOW_ARCHIVES_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcBlockTypesOptions
///
/// Clap-parsed options for generic ICRC-3 supported block type queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBlockTypesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcBlockTypesOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches_or_usage(icrc_block_types_command(), args, icrc_block_types_usage)
                .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcArchivesOptions
///
/// Clap-parsed options for generic ICRC-3 archive range queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcArchivesOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) from_canister_id: Option<String>,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcArchivesOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(icrc_archives_command(), args, icrc_archives_usage)
            .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            from_canister_id: string_option(&matches, FROM_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

///
/// IcrcTipCertificateOptions
///
/// Clap-parsed options for generic ICRC-3 tip certificate queries.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTipCertificateOptions {
    pub(in crate::icrc) ledger_canister_id: String,
    pub(in crate::icrc) format: OutputFormat,
    pub(in crate::icrc) source_endpoint: String,
}

impl IcrcTipCertificateOptions {
    fn parse<I>(args: I) -> Result<Self, IcrcError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(
            icrc_tip_certificate_command(),
            args,
            icrc_tip_certificate_usage,
        )
        .map_err(IcrcError::Usage)?;
        Ok(Self {
            ledger_canister_id: required_string(&matches, LEDGER_CANISTER_ID_ARG),
            format: format_from_matches(&matches),
            source_endpoint: source_endpoint_from_matches(&matches),
        })
    }
}

fn ledger_canister_id_arg() -> clap::Arg {
    principal_arg(LEDGER_CANISTER_ID_ARG, "ICRC ledger canister principal")
}

fn with_common_icrc_options(command: ClapCommand) -> ClapCommand {
    with_icrc_format_option(with_icrc_source_endpoint_option(command))
}

fn with_icrc_source_endpoint_option(command: ClapCommand) -> ClapCommand {
    command.arg(icrc_source_endpoint_arg())
}

fn with_icrc_format_option(command: ClapCommand) -> ClapCommand {
    command.arg(format_arg())
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
        IcrcAllowanceOptions, IcrcArchivesOptions, IcrcBalanceOptions, IcrcBlockTypesOptions,
        IcrcCapabilitiesOptions, IcrcIndexOptions, IcrcTipCertificateOptions, IcrcTokenOptions,
        IcrcTransactionsOptions, icrc_allowance_usage, icrc_archives_usage, icrc_balance_usage,
        icrc_block_types_usage, icrc_capabilities_usage, icrc_index_usage,
        icrc_tip_certificate_usage, icrc_token_usage, icrc_transactions_usage, usage,
    };

    pub(in crate::icrc) fn parse_token_options(args: &[&str]) -> IcrcTokenOptions {
        try_parse_token_options(args).expect("parse ICRC token options")
    }

    pub(in crate::icrc) fn try_parse_token_options(
        args: &[&str],
    ) -> Result<IcrcTokenOptions, crate::icrc::model::IcrcError> {
        IcrcTokenOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_capabilities_options(args: &[&str]) -> IcrcCapabilitiesOptions {
        try_parse_capabilities_options(args).expect("parse ICRC capabilities options")
    }

    pub(in crate::icrc) fn try_parse_capabilities_options(
        args: &[&str],
    ) -> Result<IcrcCapabilitiesOptions, crate::icrc::model::IcrcError> {
        IcrcCapabilitiesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
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

    pub(in crate::icrc) fn parse_block_types_options(args: &[&str]) -> IcrcBlockTypesOptions {
        try_parse_block_types_options(args).expect("parse ICRC block types options")
    }

    pub(in crate::icrc) fn try_parse_block_types_options(
        args: &[&str],
    ) -> Result<IcrcBlockTypesOptions, crate::icrc::model::IcrcError> {
        IcrcBlockTypesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_archives_options(args: &[&str]) -> IcrcArchivesOptions {
        try_parse_archives_options(args).expect("parse ICRC archives options")
    }

    pub(in crate::icrc) fn try_parse_archives_options(
        args: &[&str],
    ) -> Result<IcrcArchivesOptions, crate::icrc::model::IcrcError> {
        IcrcArchivesOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn parse_tip_certificate_options(
        args: &[&str],
    ) -> IcrcTipCertificateOptions {
        try_parse_tip_certificate_options(args).expect("parse ICRC tip certificate options")
    }

    pub(in crate::icrc) fn try_parse_tip_certificate_options(
        args: &[&str],
    ) -> Result<IcrcTipCertificateOptions, crate::icrc::model::IcrcError> {
        IcrcTipCertificateOptions::parse(args.iter().copied().map(std::ffi::OsString::from))
    }

    pub(in crate::icrc) fn root_usage() -> String {
        usage()
    }

    pub(in crate::icrc) fn token_usage() -> String {
        icrc_token_usage()
    }

    pub(in crate::icrc) fn capabilities_usage() -> String {
        icrc_capabilities_usage()
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

    pub(in crate::icrc) fn block_types_usage() -> String {
        icrc_block_types_usage()
    }

    pub(in crate::icrc) fn archives_usage() -> String {
        icrc_archives_usage()
    }

    pub(in crate::icrc) fn tip_certificate_usage() -> String {
        icrc_tip_certificate_usage()
    }
}
