//! Module: sns::commands::spec::commands::lookup
//!
//! Responsibility: build clap specs for SNS lookup-style live commands.
//! Does not own: option parsing, report building, or source calls.
//! Boundary: defines shared lookup command shape and examples.

use crate::{
    cli::common::{COLLECTION_MODE_LIVE, collection_help, json_arg, source_endpoint_arg},
    sns::commands::spec::commands::args::sns_lookup_input_arg,
};
use clap::Command as ClapCommand;
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

const SNS_INFO_HELP_AFTER: &str = "\
Examples:
  icq sns info 1
  icq sns info 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns info 1 --json";

const SNS_TOKEN_HELP_AFTER: &str = "\
Examples:
  icq sns token 1
  icq sns token 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns token 1 --json";

const SNS_PARAMS_HELP_AFTER: &str = "\
Examples:
  icq sns params 1
  icq sns params 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns params 1 --json";

const SNS_SWAP_HELP_AFTER: &str = "\
Queries exactly three bounded native swap methods; does not call get_state,
enumerate participants, or create a cache.

Examples:
  icq sns swap 1
  icq sns swap 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns swap 1 --json";

const SNS_UPGRADE_HELP_AFTER: &str = "\
Uses Governance get_running_sns_version and SNS-W get_next_sns_version.
Including bounded discovery, performs at most four live calls; does not read the
upgrade journal, download Wasms, fan out, or create a cache.

Examples:
  icq sns upgrade 1
  icq sns upgrade 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns upgrade 1 --json";

pub(in crate::sns::commands) fn sns_info_command() -> ClapCommand {
    sns_lookup_command(
        "info",
        "icq sns info",
        "Resolve a deployed SNS by list id or root principal",
        "IC API endpoint used for SNS-W and governance metadata queries",
        SNS_INFO_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_token_command() -> ClapCommand {
    sns_lookup_command(
        "token",
        "icq sns token",
        "Show SNS ledger token metadata by list id or root principal",
        "IC API endpoint used for SNS-W, governance, and ledger queries",
        SNS_TOKEN_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_params_command() -> ClapCommand {
    sns_lookup_command(
        "params",
        "icq sns params",
        "Show SNS governance nervous system parameters by list id or root principal",
        "IC API endpoint used for SNS-W and governance queries",
        SNS_PARAMS_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_swap_command() -> ClapCommand {
    sns_lookup_command(
        "swap",
        "icq sns swap",
        "Show bounded SNS swap lifecycle, sale parameters, and derived state",
        "IC API endpoint used for SNS-W, Governance metadata, and swap queries",
        SNS_SWAP_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_upgrade_command() -> ClapCommand {
    sns_lookup_command(
        "upgrade",
        "icq sns upgrade",
        "Show the running SNS version and next blessed upgrade",
        "IC API endpoint used for SNS-W and Governance metadata/version queries",
        SNS_UPGRADE_HELP_AFTER,
    )
}

pub(super) fn sns_lookup_command(
    name: &'static str,
    bin_name: &'static str,
    about: &'static str,
    source_endpoint_help: &'static str,
    after_help: &'static str,
) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(bin_name)
        .about(about)
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
        .arg(source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT).help(source_endpoint_help))
        .after_help(collection_help(COLLECTION_MODE_LIVE, after_help))
}
