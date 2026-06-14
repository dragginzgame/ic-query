use super::common::command_unix_secs;
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    sns::{
        commands::{
            SnsCommandError,
            options::SnsLookupOptions,
            spec::{
                sns_info_command, sns_info_usage, sns_params_command, sns_params_usage,
                sns_token_command, sns_token_usage,
            },
        },
        report::{
            SnsHostError, SnsLookupRequest, build_sns_info_report, build_sns_params_report,
            build_sns_token_report, sns_info_report_text, sns_params_report_text,
            sns_token_report_text,
        },
    },
    version_text,
};
use clap::Command as ClapCommand;
use serde::Serialize;
use std::ffi::OsString;

pub(super) fn run_sns_info<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_info_command,
        sns_info_usage,
        build_sns_info_report,
        sns_info_report_text,
    )
}

pub(super) fn run_sns_token<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_token_command,
        sns_token_usage,
        build_sns_token_report,
        sns_token_report_text,
    )
}

pub(super) fn run_sns_params<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_params_command,
        sns_params_usage,
        build_sns_params_report,
        sns_params_report_text,
    )
}

fn run_sns_lookup<I, Report>(
    args: I,
    command: fn() -> ClapCommand,
    usage: fn() -> String,
    build_report: fn(&SnsLookupRequest) -> Result<Report, SnsHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Report: Serialize,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let options = SnsLookupOptions::parse(args, command, usage)?;
    let format = options.format;
    let request = sns_lookup_request(options)?;
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}

fn sns_lookup_request(options: SnsLookupOptions) -> Result<SnsLookupRequest, SnsCommandError> {
    Ok(SnsLookupRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.input,
    })
}
