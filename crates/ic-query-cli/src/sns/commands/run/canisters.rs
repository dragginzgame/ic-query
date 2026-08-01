//! Module: sns::commands::run::canisters
//!
//! Responsibility: dispatch SNS canister inventory and health commands.
//! Does not own: Root transport, report construction, clap specs, or rendering.
//! Boundary: routes nested canister commands through the shared lookup runner.

use crate::sns::commands::SnsCommandError;
use clap::ArgMatches;
use ic_query::sns::{build_sns_canister_report, sns_canister_report_text};
pub(super) fn run_sns_canister(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => super::lookup::run_sns_lookup(
            matches,
            network,
            build_sns_canister_report,
            sns_canister_report_text,
        ),
        _ => unreachable!("clap requires a known SNS canister subcommand"),
    }
}
