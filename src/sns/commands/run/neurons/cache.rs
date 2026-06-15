use super::super::common::command_icp_root;
use crate::{
    cli::{
        clap::parse_required_subcommand, common::write_text_or_json, help::print_help_or_version,
    },
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
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_sns_neurons_cache<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(sns_neurons_cache_command(), args)
        .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_usage()))?;
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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_list_usage, version_text()) {
        return Ok(());
    }
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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_status_usage, version_text()) {
        return Ok(());
    }
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
