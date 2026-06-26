//! Module: nns::proposals::run
//!
//! Responsibility: run NNS proposal commands.
//! Does not own: clap specs, report construction internals, or text rendering details.
//! Boundary: maps parsed options into report requests and writes text or JSON output.

use super::{
    options::{
        NnsProposalCacheOptions, NnsProposalListOptions, NnsProposalOptions,
        NnsProposalRefreshOptions,
    },
    report::{
        NnsProposalCacheListRequest, NnsProposalCacheStatusRequest, NnsProposalListRequest,
        NnsProposalRefreshRequest, NnsProposalRequest, build_nns_proposal_cache_list_report,
        build_nns_proposal_cache_status_report, build_nns_proposal_list_report,
        build_nns_proposal_list_report_from_cache, build_nns_proposal_report,
        build_nns_proposal_report_from_cache, nns_proposal_cache_list_report_text,
        nns_proposal_cache_status_report_text, nns_proposal_list_report_text,
        nns_proposal_refresh_report_text, nns_proposal_report_text, refresh_nns_proposal_cache,
    },
};
use crate::{
    cli::common::write_text_or_json,
    nns::{
        NnsCommandError, command_args, command_icp_root, now_unix_secs,
        parse_nns_required_subcommand,
    },
};
use std::ffi::OsString;

const PROPOSAL_CACHE_COMMAND: &str = "cache";
const PROPOSAL_CACHE_LIST_COMMAND: &str = "list";
const PROPOSAL_CACHE_STATUS_COMMAND: &str = "status";
const PROPOSAL_INFO_COMMAND: &str = "info";
const PROPOSAL_LIST_COMMAND: &str = "list";
const PROPOSAL_REFRESH_COMMAND: &str = "refresh";

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_nns_proposal(args)
}

fn run_nns_proposal_list<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposal_list_usage_for_error) else {
        return Ok(());
    };
    let options = NnsProposalListOptions::parse_list(args)?;
    run_nns_proposal_list_with_options(options)
}

fn run_nns_proposal_list_with_options(
    options: NnsProposalListOptions,
) -> Result<(), NnsCommandError> {
    let mut request = NnsProposalListRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.limit,
    )
    .with_status(options.status)
    .with_reward_status(options.reward_status)
    .with_topic(options.topic)
    .with_sort(options.sort)
    .with_sort_direction(options.sort_direction)
    .with_verbose(options.verbose);

    if let Some(before_proposal_id) = options.before_proposal_id {
        request = request.with_before_proposal_id(before_proposal_id);
    }
    if let Some(proposer_neuron_id) = options.proposer_neuron_id {
        request = request.with_proposer_neuron_id(proposer_neuron_id);
    }
    if let Some(query) = options.query {
        request = request.with_query(query);
    }

    let report = build_nns_proposal_list_report_from_cache(&request, &command_icp_root()?)?
        .map_or_else(|| build_nns_proposal_list_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_proposal_list_report_text)
}

fn run_nns_proposal<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposal_usage_for_error) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(
        super::commands::nns_proposal_command(),
        args,
        super::commands::nns_proposal_usage_for_error,
    )?;
    match command.as_str() {
        PROPOSAL_CACHE_COMMAND => run_nns_proposal_cache(args),
        PROPOSAL_LIST_COMMAND => run_nns_proposal_list(args),
        PROPOSAL_INFO_COMMAND => run_nns_proposal_info(args),
        PROPOSAL_REFRESH_COMMAND => run_nns_proposal_refresh(args),
        _ => unreachable!("nns proposal dispatch only defines known commands"),
    }
}

fn run_nns_proposal_info<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposal_info_usage_for_error) else {
        return Ok(());
    };
    let options = NnsProposalOptions::parse_info(args)?;
    run_nns_proposal_with_options(options)
}

fn run_nns_proposal_with_options(options: NnsProposalOptions) -> Result<(), NnsCommandError> {
    let request = NnsProposalRequest::new(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.proposal_id,
    )
    .with_show_ballots(options.show_ballots)
    .with_verbose(options.verbose);
    let report = build_nns_proposal_report_from_cache(&request, &command_icp_root()?)?
        .map_or_else(|| build_nns_proposal_report(&request), Ok)?;
    write_text_or_json(options.format, &report, nns_proposal_report_text)
}

fn run_nns_proposal_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposal_refresh_usage_for_error)
    else {
        return Ok(());
    };
    let options = NnsProposalRefreshOptions::parse(args)?;
    let request = NnsProposalRefreshRequest::new(
        command_icp_root()?,
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.page_size,
    )
    .with_max_pages(options.max_pages);
    let report = refresh_nns_proposal_cache(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_refresh_report_text)
}

fn run_nns_proposal_cache<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposal_cache_usage_for_error) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(
        super::commands::nns_proposal_cache_command(),
        args,
        super::commands::nns_proposal_cache_usage_for_error,
    )?;
    match command.as_str() {
        PROPOSAL_CACHE_LIST_COMMAND => run_nns_proposal_cache_list(args),
        PROPOSAL_CACHE_STATUS_COMMAND => run_nns_proposal_cache_status(args),
        _ => unreachable!("nns proposal cache dispatch only defines known commands"),
    }
}

fn run_nns_proposal_cache_list<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(
        args,
        super::commands::nns_proposal_cache_list_usage_for_error,
    ) else {
        return Ok(());
    };
    let options = NnsProposalCacheOptions::parse_list(args)?;
    let request = NnsProposalCacheListRequest::new(command_icp_root()?, options.network);
    let report = build_nns_proposal_cache_list_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_cache_list_report_text)
}

fn run_nns_proposal_cache_status<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(
        args,
        super::commands::nns_proposal_cache_status_usage_for_error,
    ) else {
        return Ok(());
    };
    let options = NnsProposalCacheOptions::parse_status(args)?;
    let request = NnsProposalCacheStatusRequest::new(command_icp_root()?, options.network);
    let report = build_nns_proposal_cache_status_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        nns_proposal_cache_status_report_text,
    )
}
