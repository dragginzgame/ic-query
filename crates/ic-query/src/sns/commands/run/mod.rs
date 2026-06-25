//! Module: sns::commands::run
//!
//! Responsibility: dispatch parsed SNS command families into report builders.
//! Does not own: clap command shape, report construction, or text rendering.
//! Boundary: maps command-line options into report requests.

mod common;
mod lookup;
mod neurons;
mod proposals;

use crate::{
    cli::common::write_text_or_json,
    sns::{
        commands::{
            SnsCommandError,
            options::SnsListOptions,
            run::common::{command_args, command_unix_secs, parse_required_command},
            spec::{sns_command, sns_list_usage, usage},
        },
        report::{SnsListRequest, build_sns_list_report, sns_list_report_text},
    },
};
use std::ffi::OsString;

pub fn run<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_command(sns_command(), args, usage)?;

    match command.as_str() {
        "list" => run_sns_list(args),
        "info" => lookup::run_sns_info(args),
        "token" => lookup::run_sns_token(args),
        "params" => lookup::run_sns_params(args),
        "proposal" => proposals::run_sns_proposal(args),
        "proposals" => proposals::run_sns_proposals(args),
        "neurons" => neurons::run_sns_neurons(args),
        _ => unreachable!("sns dispatch command only defines known commands"),
    }
}

fn run_sns_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_list_usage) else {
        return Ok(());
    };
    let options = SnsListOptions::parse(args)?;
    let format = options.format;
    let request = SnsListRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let report = build_sns_list_report(&request)?;
    write_text_or_json(format, &report, sns_list_report_text)
}
