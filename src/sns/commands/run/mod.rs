mod common;
mod lookup;
mod neurons;
mod proposals;

use super::{
    SnsCommandError,
    options::SnsListOptions,
    spec::{sns_command, sns_list_usage, usage},
};
use crate::{
    cli::{
        clap::parse_required_subcommand_or_usage, common::write_text_or_json,
        help::print_help_or_version,
    },
    sns::report::{SnsListRequest, build_sns_list_report, sns_list_report_text},
    version_text,
};
use common::command_unix_secs;
use std::ffi::OsString;

pub fn run<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand_or_usage(sns_command(), args, usage)
        .map_err(SnsCommandError::Usage)?;

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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_list_usage, version_text()) {
        return Ok(());
    }
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
