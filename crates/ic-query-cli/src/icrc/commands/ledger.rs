//! Module: icrc::commands::ledger
//!
//! Responsibility: construct the ledger-wide ICRC Clap command tree.
//! Does not own: typed option extraction, command dispatch, report construction, or output.
//! Boundary: keeps metadata, capability, transaction, archive, and certificate commands together.

use super::{
    DEFAULT_ICRC_TRANSACTIONS_LIMIT, FOLLOW_ARCHIVES_ARG, FROM_CANISTER_ID_ARG, LIMIT_ARG,
    MAX_ICRC_TRANSACTIONS_LIMIT, START_ARG, ledger_canister_id_arg, principal_text_value_parser,
    with_common_icrc_options, with_icrc_json_option, with_icrc_source_endpoint_option,
};
use crate::cli::{
    clap::{flag_arg, value_arg},
    common::{COLLECTION_MODE_LIVE, collection_help},
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("ledger")
        .bin_name("icq icrc ledger")
        .about("Inspect ledger-wide ICRC metadata and transactions")
        .subcommand_required(true)
        .subcommand(icrc_capabilities_command())
        .subcommand(icrc_token_command())
        .subcommand(icrc_index_command())
        .subcommand(icrc_transactions_command())
        .subcommand(icrc_block_types_command())
        .subcommand(icrc_archives_command())
        .subcommand(icrc_tip_certificate_command())
}

pub(super) fn icrc_token_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "token",
        "icq icrc ledger token",
        "Show generic ICRC token metadata by ledger canister id",
        "Examples:\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(super) fn icrc_capabilities_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "capabilities",
        "icq icrc ledger capabilities",
        "Probe generic ICRC ledger endpoint capabilities",
        "Examples:\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

pub(super) fn icrc_index_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "index",
        "icq icrc ledger index",
        "Show a generic ICRC ledger index canister",
        "Examples:\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(super) fn icrc_transactions_command() -> ClapCommand {
    let command = ClapCommand::new("transactions")
        .bin_name("icq icrc ledger transactions")
        .about("Show a generic ICRC ledger transaction history page")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --json",
        ))
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
    with_icrc_json_option(command)
}

pub(super) fn icrc_block_types_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "block-types",
        "icq icrc ledger block-types",
        "Show generic ICRC-3 ledger supported block types",
        "Examples:\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(super) fn icrc_archives_command() -> ClapCommand {
    let command = ClapCommand::new("archives")
        .bin_name("icq icrc ledger archives")
        .about("Show generic ICRC-3 ledger archive ranges")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --json",
        ))
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

pub(super) fn icrc_tip_certificate_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "tip-certificate",
        "icq icrc ledger tip-certificate",
        "Show a generic ICRC-3 ledger tip certificate",
        "Examples:\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

fn simple_icrc_ledger_command(
    name: &'static str,
    bin_name: &'static str,
    about: &'static str,
    examples: &'static str,
) -> ClapCommand {
    let command = ClapCommand::new(name)
        .bin_name(bin_name)
        .about(about)
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
        .arg(ledger_canister_id_arg());
    with_common_icrc_options(command)
}
