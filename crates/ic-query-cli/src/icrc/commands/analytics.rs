//! Module: icrc::commands::analytics
//!
//! Responsibility: construct the bounded official ICRC analytics Clap command tree.
//! Does not own: typed option extraction, live HTTP calls, reports, or output.
//! Boundary: exposes only explicit ledger and time bounds supported by the official API.

use super::{
    ACCOUNT_ID_ARG, ledger_canister_id_arg,
    options::{AccountListOptions, HolderListOptions, TokenValueOptions, TotalSupplyOptions},
};
use crate::cli::{
    clap::value_arg,
    common::{COLLECTION_MODE_LIVE, collection_help, json_arg, source_endpoint_arg},
};
use clap::{Args, Command as ClapCommand};
use ic_query::ic::{
    DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT, DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
};

pub(in crate::icrc) fn command() -> ClapCommand {
    ClapCommand::new("analytics")
        .about("Inspect bounded official ICRC analytics")
        .subcommand(icrc_analytics_account_command())
        .subcommand(icrc_analytics_holder_command())
        .subcommand(icrc_analytics_token_values_command())
        .subcommand(icrc_analytics_total_supply_command())
        .subcommand(icrc_analytics_transaction_command())
}

fn icrc_analytics_account_command() -> ClapCommand {
    ClapCommand::new("account")
        .about("Inspect indexed account analytics for one ICRC ledger")
        .subcommand(indexed_count_command("account", "accounts"))
        .subcommand(icrc_analytics_account_info_command())
        .subcommand(icrc_analytics_account_list_command())
}

fn icrc_analytics_holder_command() -> ClapCommand {
    ClapCommand::new("holder")
        .about("Inspect indexed holder analytics for one ICRC ledger")
        .subcommand(indexed_count_command("holder", "holders"))
        .subcommand(icrc_analytics_holder_list_command())
}

fn icrc_analytics_transaction_command() -> ClapCommand {
    indexed_count_namespace("transaction", "transactions")
}

pub(in crate::icrc) fn icrc_analytics_token_values_command() -> ClapCommand {
    with_common_analytics_options(
        TokenValueOptions::augment_args(ClapCommand::new("token-values"))

            .about("Show bounded external token value observations for one ICRC ledger")
            .long_about(
                "Show a bounded token price and 24-hour volume series aggregated by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, performs no pagination or follow-up calls, and does not use a cache. Rows retain their external provider name and URL; values are off-chain and non-certified.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai --start 1785542400 --end 1785628800 --limit 100 --json",
            )),
    )
}

pub(in crate::icrc) fn icrc_analytics_total_supply_command() -> ClapCommand {
    with_common_analytics_options(
        TotalSupplyOptions::augment_args(ClapCommand::new("total-supply"))

            .about("Show bounded historical total supply for one indexed ICRC ledger")
            .long_about(
                "Show bounded historical total supply for one ledger indexed by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, performs no enumeration or follow-up calls, and does not use a cache. Values are off-chain, non-certified analytics in raw ledger base units.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai --start 1785542400 --end 1785801600 --step 86400 --json",
            )),
    )
}

fn with_common_analytics_options(command: ClapCommand) -> ClapCommand {
    with_analytics_endpoint(command, DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT)
}

fn with_analytics_endpoint(command: ClapCommand, endpoint: &'static str) -> ClapCommand {
    command
        .arg(ledger_canister_id_arg().index(1))
        .arg(
            source_endpoint_arg(endpoint)
                .help("Official IC Dashboard ICRC analytics API base endpoint"),
        )
        .arg(json_arg())
}

fn indexed_count_namespace(entity: &'static str, plural: &'static str) -> ClapCommand {
    ClapCommand::new(entity)
        .about(format!(
            "Inspect indexed {entity} analytics for one ICRC ledger"
        ))
        .subcommand(indexed_count_command(entity, plural))
}

pub(in crate::icrc) fn icrc_analytics_account_info_command() -> ClapCommand {
    with_analytics_endpoint(
        ClapCommand::new("info")

            .about("Show one exact off-chain account record; not point-in-time guaranteed")
            .long_about(
                "Show one exact account record maintained by the official IC Dashboard ICRC analytics API. The account id is the opaque stable id returned by account list. The command makes exactly one live request, performs no native ledger follow-up, and does not use a cache. Values are off-chain, non-certified, and not point-in-time guaranteed.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics account info mxzaz-hqaaa-aaaar-qaada-cai 222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae\n  icq icrc analytics account info mxzaz-hqaaa-aaaar-qaada-cai 222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae --json",
            ))
            .arg(
                value_arg(ACCOUNT_ID_ARG)
                    .value_name("account-id")
                    .index(2)
                    .required(true)
                    .help("Opaque account id returned by account list"),
            ),
        DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT,
    )
}

pub(in crate::icrc) fn icrc_analytics_account_list_command() -> ClapCommand {
    with_common_analytics_options(
        AccountListOptions::augment_args(ClapCommand::new("list"))

                .about("Show one bounded off-chain account page; not a complete snapshot")
                .long_about(
                    "Show one explicitly bounded account page maintained by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, never follows cursors automatically, and does not use a cache. Returned cursors can be supplied unchanged to --after or --before. Rows are off-chain, non-certified, and not a complete or point-in-time ledger snapshot.",
                )
                .after_help(collection_help(
                    COLLECTION_MODE_LIVE,
                    "Examples:\n  icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai --limit 25\n  icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai --sort-by=-balance --json",
                )),
    )
}

pub(in crate::icrc) fn icrc_analytics_holder_list_command() -> ClapCommand {
    with_common_analytics_options(
        HolderListOptions::augment_args(ClapCommand::new("list"))

            .about("Show one bounded off-chain holder page; not a complete snapshot")
            .long_about(
                "Show one explicitly bounded principal-level holder page maintained by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, never follows cursors automatically, and does not use a cache. Returned cursors can be supplied unchanged to --after or --before. Rows are off-chain, non-certified aggregates and not a complete or point-in-time ledger snapshot.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics holder list mxzaz-hqaaa-aaaar-qaada-cai --limit 25\n  icq icrc analytics holder list mxzaz-hqaaa-aaaar-qaada-cai --sort-by=-balance --json",
            )),
    )
}

fn indexed_count_command(entity: &'static str, plural: &'static str) -> ClapCommand {
    with_common_analytics_options(
        ClapCommand::new("count")

            .about(format!(
                "Show the current indexed {entity} count for one ICRC ledger"
            ))
            .long_about(format!(
                "Show the current {entity} count for one ledger indexed by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, requests no {plural} rows, performs no enumeration or follow-up calls, and does not use a cache. The count is off-chain and non-certified."
            ))
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                &format!(
                    "Examples:\n  icq icrc analytics {entity} count mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc analytics {entity} count mxzaz-hqaaa-aaaar-qaada-cai --json"
                ),
            )),
    )
}
