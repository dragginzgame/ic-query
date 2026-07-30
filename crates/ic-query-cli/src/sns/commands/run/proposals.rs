//! Module: sns::commands::run::proposals
//!
//! Responsibility: run SNS proposal detail, list, refresh, and cache commands.
//! Does not own: proposal cache storage, live governance calls, or rendering.
//! Boundary: maps proposal CLI options into report/cache request DTOs.

use crate::{
    cli::common::write_text_or_json,
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        options::{
            SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
            SnsProposalsOptions, SnsProposalsRefreshOptions,
        },
        run::common::{
            cache_command_parts, cached_lookup_command_parts, command_args, parse_required_command,
        },
        spec::{
            sns_proposal_cache_command, sns_proposal_cache_list_usage,
            sns_proposal_cache_status_usage, sns_proposal_cache_usage, sns_proposal_command,
            sns_proposal_info_usage, sns_proposal_list_usage, sns_proposal_refresh_usage,
            sns_proposal_usage,
        },
    },
};
use ic_query::sns::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsProposalRequest, SnsProposalsRefreshRequest,
    SnsProposalsRequest, build_sns_proposal_report, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, build_sns_proposals_report_with_progress,
    refresh_sns_proposals_cache_with_progress, sns_proposal_report_text,
    sns_proposals_cache_list_report_text, sns_proposals_cache_status_report_text,
    sns_proposals_refresh_report_text, sns_proposals_report_text,
};
use std::ffi::OsString;

pub(super) fn run_sns_proposal<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_command(sns_proposal_command(), args, sns_proposal_usage)?;
    match command.as_str() {
        "list" => run_sns_proposal_list(args),
        "info" => run_sns_proposal_info(args),
        "refresh" => run_sns_proposal_refresh(args),
        "cache" => run_sns_proposal_cache(args),
        _ => unreachable!("sns proposal dispatch command only defines known commands"),
    }
}

fn run_sns_proposal_info<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_info_usage) else {
        return Ok(());
    };
    let options = SnsProposalOptions::parse(args)?;
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        proposal_id: options.proposal_id,
        cache_root: Some(parts.cache_root),
        verbose: options.verbose,
        show_ballots: options.show_ballots,
    };
    let report = build_sns_proposal_report(&request)?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

fn run_sns_proposal_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_list_usage) else {
        return Ok(());
    };
    let options = SnsProposalsOptions::parse(args)?;
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalsRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status.into(),
        topic: options.topic.into(),
        eligibility: options.eligibility.into(),
        proposer_neuron_id: options.proposer_neuron_id,
        query: options.query,
        sort: options.sort.into(),
        sort_direction: options.sort_direction,
        cache_root: Some(parts.cache_root),
        verbose: options.verbose,
    };
    let mut progress = StderrQueryProgress::new();
    let report = build_sns_proposals_report_with_progress(&request, &mut progress)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}

fn run_sns_proposal_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_refresh_usage) else {
        return Ok(());
    };
    let options = SnsProposalsRefreshOptions::parse(args)?;
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalsRefreshRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        cache_root: parts.cache_root,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let mut progress = StderrQueryProgress::new();
    let report = refresh_sns_proposals_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(format, &report, sns_proposals_refresh_report_text)
}

fn run_sns_proposal_cache<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_cache_usage) else {
        return Ok(());
    };
    let (command, args) =
        parse_required_command(sns_proposal_cache_command(), args, sns_proposal_cache_usage)?;
    match command.as_str() {
        "list" => run_sns_proposal_cache_list(args),
        "status" => run_sns_proposal_cache_status(args),
        _ => unreachable!("sns proposal cache dispatch command only defines known commands"),
    }
}

fn run_sns_proposal_cache_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_cache_list_usage) else {
        return Ok(());
    };
    let options = SnsProposalsCacheListOptions::parse(args)?;
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheListRequest {
        network: parts.network,
        cache_root: parts.cache_root,
    };
    let report = build_sns_proposals_cache_list_report(&request)?;
    write_text_or_json(parts.format, &report, sns_proposals_cache_list_report_text)
}

fn run_sns_proposal_cache_status<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_cache_status_usage) else {
        return Ok(());
    };
    let options = SnsProposalsCacheStatusOptions::parse(args)?;
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsCacheStatusRequest {
        network: parts.network,
        cache_root: parts.cache_root,
        input: options.input,
    };
    let report = build_sns_proposals_cache_status_report(&request)?;
    write_text_or_json(
        parts.format,
        &report,
        sns_proposals_cache_status_report_text,
    )
}
