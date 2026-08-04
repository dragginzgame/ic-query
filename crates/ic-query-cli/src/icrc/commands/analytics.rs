//! Module: icrc::commands::analytics
//!
//! Responsibility: construct the bounded official ICRC analytics Clap command tree.
//! Does not own: typed option extraction, live HTTP calls, reports, or output.
//! Boundary: exposes only explicit ledger and time bounds supported by the official API.

use super::{END_ARG, START_ARG, STEP_ARG, ledger_canister_id_arg};
use crate::cli::{
    clap::value_arg,
    common::{COLLECTION_MODE_LIVE, collection_help, json_arg, source_endpoint_arg},
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT, MIN_ICRC_ANALYTICS_TIMESTAMP};

pub(in crate::icrc) fn command() -> ClapCommand {
    ClapCommand::new("analytics")
        .bin_name("icq icrc analytics")
        .about("Inspect bounded official ICRC analytics")
        .subcommand(icrc_analytics_holder_command())
        .subcommand(icrc_analytics_total_supply_command())
}

pub(in crate::icrc) fn icrc_analytics_holder_command() -> ClapCommand {
    ClapCommand::new("holder")
        .bin_name("icq icrc analytics holder")
        .about("Inspect holder analytics for one indexed ICRC ledger")
        .subcommand(icrc_analytics_holder_count_command())
}

pub(in crate::icrc) fn icrc_analytics_holder_count_command() -> ClapCommand {
    with_common_analytics_options(
        ClapCommand::new("count")
            .bin_name("icq icrc analytics holder count")
            .about("Show the current indexed holder count for one ICRC ledger")
            .long_about(
                "Show the current holder count for one ledger indexed by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, requests no holder rows, performs no enumeration or follow-up calls, and does not use a cache. The count is off-chain and non-certified.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai --json",
            )),
    )
}

pub(in crate::icrc) fn icrc_analytics_total_supply_command() -> ClapCommand {
    with_common_analytics_options(
        ClapCommand::new("total-supply")
            .bin_name("icq icrc analytics total-supply")
            .about("Show bounded historical total supply for one indexed ICRC ledger")
            .long_about(
                "Show bounded historical total supply for one ledger indexed by the official IC Dashboard ICRC analytics API. The command makes exactly one live request, performs no enumeration or follow-up calls, and does not use a cache. Values are off-chain, non-certified analytics in raw ledger base units.",
            )
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                "Examples:\n  icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai\n  icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai --start 1785542400 --end 1785801600 --step 86400 --json",
            ))
            .arg(
                value_arg(START_ARG)
                    .long(START_ARG)
                    .value_name("unix-seconds")
                    .value_parser(
                        RangedU64ValueParser::<u64>::new()
                            .range(MIN_ICRC_ANALYTICS_TIMESTAMP..),
                    )
                    .help("Inclusive series start; defaults to 30 days before --end"),
            )
            .arg(
                value_arg(END_ARG)
                    .long(END_ARG)
                    .value_name("unix-seconds")
                    .value_parser(
                        RangedU64ValueParser::<u64>::new()
                            .range(MIN_ICRC_ANALYTICS_TIMESTAMP..),
                    )
                    .help("Inclusive series end; defaults to the current time"),
            )
            .arg(
                value_arg(STEP_ARG)
                    .long(STEP_ARG)
                    .value_name("seconds")
                    .value_parser(["3600", "86400"])
                    .help("Observation interval accepted by the official API; defaults to 86400"),
            ),
    )
}

fn with_common_analytics_options(command: ClapCommand) -> ClapCommand {
    command
        .arg(ledger_canister_id_arg())
        .arg(
            source_endpoint_arg(DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT)
                .help("Official IC Dashboard ICRC analytics API base endpoint"),
        )
        .arg(json_arg())
}
