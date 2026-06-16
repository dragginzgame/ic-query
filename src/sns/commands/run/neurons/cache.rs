use super::super::common::{command_args, command_icp_root};
use crate::{
    cli::{clap::parse_required_subcommand_or_usage, common::write_text_or_json},
    sns::{
        commands::{
            SnsCommandError,
            options::{SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions},
            spec::{
                sns_neurons_cache_command, sns_neurons_cache_list_usage,
                sns_neurons_cache_status_usage, sns_neurons_cache_usage,
            },
        },
        report::{
            SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest,
            build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
            sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
        },
    },
};
use std::ffi::OsString;

pub(super) fn run_sns_neurons_cache<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_cache_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(
        sns_neurons_cache_command(),
        args,
        sns_neurons_cache_usage,
    )
    .map_err(SnsCommandError::Usage)?;
    match command.as_str() {
        "list" => run_sns_neurons_cache_list(args),
        "status" => run_sns_neurons_cache_status(args),
        _ => unreachable!("sns neurons cache dispatch command only defines known commands"),
    }
}

fn run_sns_neurons_cache_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_cache_list_usage) else {
        return Ok(());
    };
    let options = SnsNeuronsCacheListOptions::parse(args)?;
    let format = options.format;
    let request = SnsNeuronsCacheListRequest {
        network: options.network,
        icp_root: command_icp_root()?,
    };
    let report = build_sns_neurons_cache_list_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_list_report_text)
}

fn run_sns_neurons_cache_status<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_cache_status_usage) else {
        return Ok(());
    };
    let options = SnsNeuronsCacheStatusOptions::parse(args)?;
    let format = options.format;
    let request = SnsNeuronsCacheStatusRequest {
        network: options.network,
        icp_root: command_icp_root()?,
        input: options.input,
    };
    let report = build_sns_neurons_cache_status_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_status_report_text)
}
