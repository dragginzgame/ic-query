//! Module: sns::commands::spec::commands::canisters
//!
//! Responsibility: build clap specs for SNS canister inventory and health.
//! Does not own: option parsing, Root calls, report construction, or rendering.
//! Boundary: defines the singular canister command family and live-call disclosure.

use crate::{
    cli::clap::passthrough_subcommand, sns::commands::spec::commands::lookup::sns_lookup_command,
};
use clap::Command as ClapCommand;

const SNS_CANISTER_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns canister list 1
  icq sns canister list 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns canister list 1 --json

Health collection calls SNS Root's read-only get_sns_canisters_summary ingress
method with update_canister_list=false; it does not ask Root to update its
canister inventory.";

pub(in crate::sns::commands) fn sns_canister_command() -> ClapCommand {
    ClapCommand::new("canister")
        .bin_name("icq sns canister")
        .about("Inspect SNS Root canister inventory and operational health")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List SNS Root canisters and operational health"),
        ))
}

pub(in crate::sns::commands) fn sns_canister_list_command() -> ClapCommand {
    sns_lookup_command(
        "list",
        "icq sns canister list",
        "List SNS Root canisters and operational health by SNS list id or root principal",
        "IC API endpoint used for SNS-W discovery and Root inventory and health calls",
        SNS_CANISTER_LIST_HELP_AFTER,
    )
}
