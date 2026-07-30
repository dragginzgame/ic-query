//! Module: sns::commands::run::canisters
//!
//! Responsibility: dispatch SNS canister inventory and health commands.
//! Does not own: Root transport, report construction, clap specs, or rendering.
//! Boundary: routes nested canister commands through the shared lookup runner.

use crate::sns::commands::{
    SnsCommandError,
    run::common::{command_args, parse_required_command},
    spec::{
        sns_canister_command, sns_canister_list_command, sns_canister_list_usage,
        sns_canister_usage,
    },
};
use ic_query::sns::{build_sns_canister_report, sns_canister_report_text};
use std::ffi::OsString;

pub(super) fn run_sns_canister<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_canister_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_command(sns_canister_command(), args, sns_canister_usage)?;
    match command.as_str() {
        "list" => super::lookup::run_sns_lookup(
            args,
            sns_canister_list_command,
            sns_canister_list_usage,
            build_sns_canister_report,
            sns_canister_report_text,
        ),
        _ => unreachable!("sns canister dispatch command only defines known commands"),
    }
}
