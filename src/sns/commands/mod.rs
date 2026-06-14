mod options;
mod spec;

use crate::{
    cli::{
        clap::parse_required_subcommand,
        common::{current_unix_secs, write_text_or_json},
        help::print_help_or_version,
    },
    project::icp_root,
    sns::report::{
        SnsHostError, SnsListRequest, SnsLookupRequest, SnsNeuronsCacheListRequest,
        SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest, SnsNeuronsRequest, SnsNeuronsSort,
        SnsProposalRequest, SnsProposalsRequest, build_sns_info_report, build_sns_list_report,
        build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
        build_sns_neurons_report, build_sns_params_report, build_sns_proposal_report,
        build_sns_proposals_report, build_sns_token_report, refresh_sns_neurons_cache,
        sns_info_report_text, sns_list_report_text, sns_neurons_cache_list_report_text,
        sns_neurons_cache_status_report_text, sns_neurons_refresh_report_text,
        sns_neurons_report_text, sns_params_report_text, sns_proposal_report_text,
        sns_proposals_report_text, sns_token_report_text,
    },
    version_text,
};
use clap::Command as ClapCommand;
use serde::Serialize;
use std::{ffi::OsString, io, path::PathBuf};
use thiserror::Error as ThisError;

use options::{
    SnsListOptions, SnsLookupOptions, SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions,
    SnsNeuronsOptions, SnsNeuronsRefreshOptions, SnsProposalOptions, SnsProposalsOptions,
};
use spec::{
    SnsNeuronsSortArg, sns_command, sns_info_command, sns_info_usage, sns_list_usage,
    sns_neurons_cache_command, sns_neurons_cache_list_usage, sns_neurons_cache_status_usage,
    sns_neurons_cache_usage, sns_neurons_refresh_usage, sns_neurons_usage, sns_params_command,
    sns_params_usage, sns_proposal_usage, sns_proposals_usage, sns_token_command, sns_token_usage,
    usage,
};

#[cfg(test)]
use crate::cli::common::OutputFormat;

#[cfg(test)]
use crate::sns::report::DEFAULT_SNS_SOURCE_ENDPOINT;

#[cfg(test)]
use spec::{SnsListSortArg, SnsProposalStatusArg};

#[derive(Debug, ThisError)]
pub enum SnsCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] SnsHostError),

    #[error("system clock before unix epoch: {0}")]
    Clock(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(sns_command(), args)
        .map_err(|_| SnsCommandError::Usage(usage()))?;

    match command.as_str() {
        "list" => run_sns_list(args),
        "info" => run_sns_info(args),
        "token" => run_sns_token(args),
        "params" => run_sns_params(args),
        "proposal" => run_sns_proposal(args),
        "proposals" => run_sns_proposals(args),
        "neurons" => run_sns_neurons(args),
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

fn run_sns_info<I>(args: I) -> Result<(), SnsCommandError>
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

fn run_sns_token<I>(args: I) -> Result<(), SnsCommandError>
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

fn run_sns_params<I>(args: I) -> Result<(), SnsCommandError>
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

fn run_sns_proposal<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_proposal_usage, version_text()) {
        return Ok(());
    }
    let options = SnsProposalOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsProposalRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
        proposal_id: options.proposal_id,
        verbose: options.verbose,
    };
    let report = build_sns_proposal_report(&request)?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

fn run_sns_proposals<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_proposals_usage, version_text()) {
        return Ok(());
    }
    let options = SnsProposalsOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsProposalsRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status.into(),
        verbose: options.verbose,
    };
    let report = build_sns_proposals_report(&request)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
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

fn run_sns_neurons<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_usage, version_text()) {
        return Ok(());
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("refresh") {
        return run_sns_neurons_refresh(args.into_iter().skip(1));
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("cache") {
        return run_sns_neurons_cache(args.into_iter().skip(1));
    }
    let options = SnsNeuronsOptions::parse(args)?;
    if options.sort != SnsNeuronsSortArg::Api && options.owner_principal_id.is_some() {
        return Err(SnsCommandError::Usage(
            "`icq sns neurons --sort <id|stake|maturity|created>` reads the complete full-neuron cache and does not support --owner yet; use --sort api for owner-filtered live queries".to_string(),
        ));
    }
    let format = options.lookup.format;
    let icp_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        icp_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn run_sns_neurons_cache<I>(args: I) -> Result<(), SnsCommandError>
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

fn run_sns_neurons_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = SnsNeuronsRefreshOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsNeuronsRefreshRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
        icp_root: command_icp_root()?,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let report = refresh_sns_neurons_cache(&request)?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}

fn sns_lookup_request(options: SnsLookupOptions) -> Result<SnsLookupRequest, SnsCommandError> {
    Ok(SnsLookupRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.input,
    })
}

fn command_unix_secs() -> Result<u64, SnsCommandError> {
    current_unix_secs().map_err(SnsCommandError::Clock)
}

fn command_icp_root() -> Result<PathBuf, SnsCommandError> {
    icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}

fn cache_root_for_sort(sort: SnsNeuronsSortArg) -> Result<Option<PathBuf>, SnsCommandError> {
    if SnsNeuronsSort::from(sort).uses_cache() {
        return Ok(Some(command_icp_root()?));
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
