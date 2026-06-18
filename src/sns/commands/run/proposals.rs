//! Module: sns::commands::run::proposals
//!
//! Responsibility: run SNS proposal detail, list, refresh, and cache commands.
//! Does not own: proposal cache storage, live governance calls, or rendering.
//! Boundary: maps proposal CLI options into report/cache request DTOs.

use crate::{
    cli::common::{OutputFormat, write_text_or_json},
    sns::{
        commands::{
            SnsCommandError,
            options::{
                SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
                SnsProposalsOptions, SnsProposalsRefreshOptions,
            },
            run::common::{
                command_args, command_icp_root, lookup_command_parts, parse_required_command,
            },
            spec::{
                sns_proposal_usage, sns_proposals_cache_command, sns_proposals_cache_list_usage,
                sns_proposals_cache_status_usage, sns_proposals_cache_usage,
                sns_proposals_refresh_usage, sns_proposals_usage,
            },
        },
        report::{
            SnsProposalRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
            SnsProposalsRefreshRequest, SnsProposalsRequest, build_sns_proposal_report,
            build_sns_proposals_cache_list_report, build_sns_proposals_cache_status_report,
            build_sns_proposals_report, refresh_sns_proposals_cache, sns_proposal_report_text,
            sns_proposals_cache_list_report_text, sns_proposals_cache_status_report_text,
            sns_proposals_refresh_report_text, sns_proposals_report_text,
        },
    },
};
use std::{ffi::OsString, path::PathBuf};

struct SnsProposalsCacheCommandParts {
    format: OutputFormat,
    network: String,
    icp_root: PathBuf,
}

pub(super) fn run_sns_proposal<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposal_usage) else {
        return Ok(());
    };
    let options = SnsProposalOptions::parse(args)?;
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        proposal_id: options.proposal_id,
        icp_root: Some(command_icp_root()?),
        verbose: options.verbose,
        show_ballots: options.show_ballots,
    };
    let report = build_sns_proposal_report(&request)?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

pub(super) fn run_sns_proposals<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposals_usage) else {
        return Ok(());
    };
    if args.first().and_then(|arg| arg.to_str()) == Some("refresh") {
        return run_sns_proposals_refresh(args.into_iter().skip(1));
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("cache") {
        return run_sns_proposals_cache(args.into_iter().skip(1));
    }
    let options = SnsProposalsOptions::parse(args)?;
    let parts = lookup_command_parts(options.lookup)?;
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
        sort: options.sort.into(),
        icp_root: Some(command_icp_root()?),
        verbose: options.verbose,
    };
    let report = build_sns_proposals_report(&request)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}

fn run_sns_proposals_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposals_refresh_usage) else {
        return Ok(());
    };
    let options = SnsProposalsRefreshOptions::parse(args)?;
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsProposalsRefreshRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        icp_root: command_icp_root()?,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let report = refresh_sns_proposals_cache(&request)?;
    write_text_or_json(format, &report, sns_proposals_refresh_report_text)
}

fn run_sns_proposals_cache<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposals_cache_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_command(
        sns_proposals_cache_command(),
        args,
        sns_proposals_cache_usage,
    )?;
    match command.as_str() {
        "list" => run_sns_proposals_cache_list(args),
        "status" => run_sns_proposals_cache_status(args),
        _ => unreachable!("sns proposals cache dispatch command only defines known commands"),
    }
}

fn cache_command_parts(
    format: OutputFormat,
    network: String,
) -> Result<SnsProposalsCacheCommandParts, SnsCommandError> {
    Ok(SnsProposalsCacheCommandParts {
        format,
        network,
        icp_root: command_icp_root()?,
    })
}

fn run_sns_proposals_cache_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposals_cache_list_usage) else {
        return Ok(());
    };
    let options = SnsProposalsCacheListOptions::parse(args)?;
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsProposalsCacheListRequest {
        network: parts.network,
        icp_root: parts.icp_root,
    };
    let report = build_sns_proposals_cache_list_report(&request)?;
    write_text_or_json(parts.format, &report, sns_proposals_cache_list_report_text)
}

fn run_sns_proposals_cache_status<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_proposals_cache_status_usage) else {
        return Ok(());
    };
    let options = SnsProposalsCacheStatusOptions::parse(args)?;
    let parts = cache_command_parts(options.format, options.network)?;
    let request = SnsProposalsCacheStatusRequest {
        network: parts.network,
        icp_root: parts.icp_root,
        input: options.input,
    };
    let report = build_sns_proposals_cache_status_report(&request)?;
    write_text_or_json(
        parts.format,
        &report,
        sns_proposals_cache_status_report_text,
    )
}
