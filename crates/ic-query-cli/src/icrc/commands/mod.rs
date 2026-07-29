//! Module: icrc::commands
//!
//! Responsibility: parse and run generic ICRC CLI commands.
//! Does not own: live ledger calls, report DTOs, or text rendering.
//! Boundary: maps clap options into report requests and writes text/JSON output.

mod dispatch;
mod options;
#[cfg(test)]
pub(in crate::icrc) mod test_support;

pub use dispatch::run;
use options::{
    IcrcAllowanceOptions, IcrcArchivesOptions, IcrcBalanceOptions, IcrcBlockTypesOptions,
    IcrcCapabilitiesOptions, IcrcIndexOptions, IcrcTipCertificateOptions, IcrcTokenOptions,
    IcrcTransactionsOptions,
};

use crate::cli::{
    clap::{
        flag_arg, passthrough_subcommand, render_help, required_string, required_typed, value_arg,
    },
    common::{
        COLLECTION_MODE_LIVE, OutputFormat, collection_help, format_arg, source_endpoint_arg,
    },
};
use candid::Principal;
use clap::{
    ArgMatches, Command as ClapCommand,
    builder::{RangedU64ValueParser, ValueParser},
};
use ic_query::icrc::{DEFAULT_ICRC_SOURCE_ENDPOINT, normalize_subaccount_hex};

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
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc token ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        ))
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_capabilities_command() -> ClapCommand {
    let command = ClapCommand::new("capabilities")
        .bin_name("icq icrc capabilities")
        .about("Probe generic ICRC ledger endpoint capabilities")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc capabilities mxzaz-hqaaa-aaaar-qaada-cai --format json",
        ))
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_balance_command() -> ClapCommand {
    let command = ClapCommand::new("balance")
        .bin_name("icq icrc balance")
        .about("Show a generic ICRC account balance")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa\n  icq icrc balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ))
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
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa\n  icq icrc allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000",
        ))
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
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc index ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        ))
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_transactions_command() -> ClapCommand {
    let command = ClapCommand::new("transactions")
        .bin_name("icq icrc transactions")
        .about("Show a generic ICRC ledger transaction history page")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --format json",
        ))
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
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc block-types ryjl3-tyaaa-aaaaa-aaaba-cai --format json",
        ))
        .disable_help_flag(true)
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}

fn icrc_archives_command() -> ClapCommand {
    let command = ClapCommand::new("archives")
        .bin_name("icq icrc archives")
        .about("Show generic ICRC-3 ledger archive ranges")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --format json",
        ))
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
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --format json",
        ))
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
    required_typed(matches, FORMAT_ARG)
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
