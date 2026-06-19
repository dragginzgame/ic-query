//! Module: nns::proposals::run
//!
//! Responsibility: run NNS proposal commands.
//! Does not own: clap specs, report construction internals, or text rendering details.
//! Boundary: maps parsed options into report requests and writes text or JSON output.

use super::{
    options::{NnsProposalOptions, NnsProposalsOptions},
    report::{
        NnsProposalRequest, NnsProposalsRequest, build_nns_proposal_report,
        build_nns_proposals_report, nns_proposal_report_text, nns_proposals_report_text,
    },
};
use crate::{
    cli::common::write_text_or_json,
    nns::{NnsCommandError, command_args, now_unix_secs, parse_nns_required_subcommand},
};
use std::ffi::OsString;

const PROPOSAL_INFO_COMMAND: &str = "info";
const PROPOSAL_LIST_COMMAND: &str = "list";

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
    let options = NnsProposalsOptions::parse_list(args)?;
    run_nns_proposals_with_options(options)
}

fn run_nns_proposals_with_options(options: NnsProposalsOptions) -> Result<(), NnsCommandError> {
    let request = NnsProposalsRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status,
        reward_status: options.reward_status,
        topic: options.topic,
        sort: options.sort,
        sort_direction: options.sort_direction,
        verbose: options.verbose,
    };
    let report = build_nns_proposals_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposals_report_text)
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
        PROPOSAL_LIST_COMMAND => run_nns_proposal_list(args),
        PROPOSAL_INFO_COMMAND => run_nns_proposal_info(args),
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
    let request = NnsProposalRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        proposal_id: options.proposal_id,
        show_ballots: options.show_ballots,
        verbose: options.verbose,
    };
    let report = build_nns_proposal_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_report_text)
}
