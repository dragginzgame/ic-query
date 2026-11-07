//! Module: icrc::commands::ledger
//!
//! Responsibility: construct the ledger-wide ICRC Clap command tree.
//! Does not own: typed option extraction, command dispatch, report construction, or output.
//! Boundary: keeps metadata, capability, transaction, archive, and certificate commands together.

use super::with_ledger_options as with_options;
use crate::cli::common::{COLLECTION_MODE_LIVE, collection_help};
use clap::Command as ClapCommand;

pub(in crate::icrc) fn command() -> ClapCommand {
    ClapCommand::new("ledger")
        .about("Inspect ledger-wide ICRC metadata and transactions")
        .subcommand(icrc_capabilities_command())
        .subcommand(icrc_token_command())
        .subcommand(icrc_index_command())
        .subcommand(icrc_transactions_command())
        .subcommand(icrc_block_types_command())
        .subcommand(icrc_archives_command())
        .subcommand(icrc_tip_certificate_command())
}

pub(in crate::icrc) fn icrc_token_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "token",
        "Show generic ICRC token metadata by ledger canister id",
        "Examples:\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(in crate::icrc) fn icrc_capabilities_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "capabilities",
        "Probe generic ICRC ledger endpoint capabilities",
        "Examples:\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

pub(in crate::icrc) fn icrc_index_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "index",
        "Show a generic ICRC ledger index canister",
        "Examples:\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(in crate::icrc) fn icrc_transactions_command() -> ClapCommand {
    with_options::<super::options::IcrcTransactionsOptions>(ClapCommand::new("transactions")

        .about("Show a generic ICRC ledger transaction history page")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives --json",
        ))


        ).mut_arg("source-endpoint", |arg| arg.display_order(20)).mut_arg("follow-archives", |arg| arg.display_order(21)).mut_arg("json", |arg| arg.display_order(22))
}

pub(in crate::icrc) fn icrc_block_types_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "block-types",
        "Show generic ICRC-3 ledger supported block types",
        "Examples:\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai --json",
    )
}

pub(in crate::icrc) fn icrc_archives_command() -> ClapCommand {
    with_options::<super::options::IcrcArchivesOptions>(ClapCommand::new("archives")

        .about("Show generic ICRC-3 ledger archive ranges")
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            "Examples:\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai\n  icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --json",
        ))

        ).mut_arg("source-endpoint", |arg| arg.display_order(20)).mut_arg("json", |arg| arg.display_order(22))
}

pub(in crate::icrc) fn icrc_tip_certificate_command() -> ClapCommand {
    simple_icrc_ledger_command(
        "tip-certificate",
        "Show a generic ICRC-3 ledger tip certificate",
        "Examples:\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai --json",
    )
}

fn simple_icrc_ledger_command(
    name: &'static str,
    about: &'static str,
    examples: &'static str,
) -> ClapCommand {
    with_options::<super::options::IcrcLedgerOptions>(
        ClapCommand::new(name)
            .about(about)
            .after_help(collection_help(COLLECTION_MODE_LIVE, examples)),
    )
    .mut_arg("source-endpoint", |arg| arg.display_order(20))
    .mut_arg("json", |arg| arg.display_order(22))
}
