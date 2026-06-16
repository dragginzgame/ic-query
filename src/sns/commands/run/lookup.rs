use super::common::{command_args, lookup_command_parts};
use crate::{
    cli::common::write_text_or_json,
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
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let options = SnsLookupOptions::parse(args, command, usage)?;
    let parts = lookup_command_parts(options)?;
    let format = parts.format;
    let request = SnsLookupRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
    };
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
