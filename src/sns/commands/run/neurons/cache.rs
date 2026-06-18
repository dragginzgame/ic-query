//! Module: sns::commands::run::neurons::cache
//!
//! Responsibility: run local SNS neuron cache inspection subcommands.
//! Does not own: snapshot discovery, status report construction, or text rendering.
//! Boundary: maps cache CLI options into cache-list and cache-status requests.

use crate::{
    cli::common::write_text_or_json,
    sns::{
        commands::{
            SnsCommandError,
            options::{SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions},
            run::common::{cache_command_parts, command_args, parse_required_command},
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
    let (command, args) =
        parse_required_command(sns_neurons_cache_command(), args, sns_neurons_cache_usage)?;
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
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsNeuronsCacheListRequest {
        network: parts.network,
        icp_root: parts.icp_root,
    };
    let report = build_sns_neurons_cache_list_report(&request)?;
    write_text_or_json(parts.format, &report, sns_neurons_cache_list_report_text)
}

fn run_sns_neurons_cache_status<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_cache_status_usage) else {
        return Ok(());
    };
    let options = SnsNeuronsCacheStatusOptions::parse(args)?;
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsNeuronsCacheStatusRequest {
        network: parts.network,
        icp_root: parts.icp_root,
        input: options.input,
    };
    let report = build_sns_neurons_cache_status_report(&request)?;
    write_text_or_json(parts.format, &report, sns_neurons_cache_status_report_text)
}
