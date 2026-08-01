//! Runtime dispatch for public NNS neuron commands.

use super::{
    commands::{
        neuron_cache_command, neuron_cache_status_usage_for_error, neuron_cache_usage_for_error,
        neuron_command, neuron_info_usage_for_error, neuron_list_usage_for_error,
        neuron_refresh_usage_for_error, neuron_usage_for_error,
    },
    options::{
        NnsNeuronCacheOptions, NnsNeuronInfoOptions, NnsNeuronListOptions, NnsNeuronRefreshOptions,
    },
};
use crate::{
    nns::{
        NnsCommandError, command_args, command_cache_root, now_unix_secs,
        parse_nns_required_subcommand, write_text_or_json,
    },
    progress::StderrQueryProgress,
};
use ic_query::nns::neuron::{
    NnsNeuronInfoRequest, NnsNeuronListRequest, build_nns_neuron_cache_status_report,
    build_nns_neuron_info_report, build_nns_neuron_info_report_from_cache,
    build_nns_neuron_list_report, build_nns_neuron_list_report_from_cache,
    nns_neuron_cache_status_report_text, nns_neuron_info_report_text, nns_neuron_list_report_text,
    nns_neuron_refresh_report_text, refresh_nns_neuron_cache_with_progress,
};
use ic_query::nns::{NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_usage_for_error) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(neuron_command(), args)?;
    match command.as_str() {
        "list" => run_list(args),
        "info" => run_info(args),
        "refresh" => run_refresh(args),
        "cache" => run_cache(args),
        _ => unreachable!("nns neuron dispatch only defines known commands"),
    }
}

fn run_list<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_list_usage_for_error) else {
        return Ok(());
    };
    let options = NnsNeuronListOptions::parse(args)?;
    let mut request = NnsNeuronListRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.limit,
    )
    .with_verbose(options.verbose);
    if let Some(start_neuron_id) = options.start_neuron_id {
        request = request.with_exclusive_start_neuron_id(start_neuron_id);
    }
    let report = build_nns_neuron_list_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_neuron_list_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_neuron_list_report_text)
}

fn run_info<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_info_usage_for_error) else {
        return Ok(());
    };
    let options = NnsNeuronInfoOptions::parse(args)?;
    let request = NnsNeuronInfoRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.neuron_id,
    )
    .with_verbose(options.verbose);
    let report = build_nns_neuron_info_report_from_cache(&request, &command_cache_root()?)?
        .map_or_else(|| build_nns_neuron_info_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_neuron_info_report_text)
}

fn run_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_refresh_usage_for_error) else {
        return Ok(());
    };
    let options = NnsNeuronRefreshOptions::parse(args)?;
    let request = NnsGovernanceRefreshRequest::new(
        command_cache_root()?,
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.page_size,
    )
    .with_max_pages(options.max_pages);
    let mut progress = StderrQueryProgress::new();
    let report = refresh_nns_neuron_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(options.format, &report, nns_neuron_refresh_report_text)
}

fn run_cache<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_cache_usage_for_error) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(neuron_cache_command(), args)?;
    match command.as_str() {
        "status" => run_cache_status(args),
        _ => unreachable!("nns neuron cache dispatch only defines known commands"),
    }
}

fn run_cache_status<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, neuron_cache_status_usage_for_error) else {
        return Ok(());
    };
    let options = NnsNeuronCacheOptions::parse(args)?;
    let request = NnsGovernanceCacheRequest::new(command_cache_root()?, options.network);
    let report = build_nns_neuron_cache_status_report(&request)?;
    write_text_or_json(options.format, &report, nns_neuron_cache_status_report_text)
}
