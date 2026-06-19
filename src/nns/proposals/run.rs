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
    nns::{NnsCommandError, command_args, now_unix_secs},
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(command: &str, args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    match command {
        "proposals" => run_nns_proposals(args),
        "proposal" => run_nns_proposal(args),
        _ => unreachable!("nns dispatch only routes proposal commands here"),
    }
}

fn run_nns_proposals<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, super::commands::nns_proposals_usage_for_error) else {
        return Ok(());
    };
    let options = NnsProposalsOptions::parse(args)?;
    let request = NnsProposalsRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status,
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
    let options = NnsProposalOptions::parse(args)?;
    let request = NnsProposalRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        proposal_id: options.proposal_id,
    };
    let report = build_nns_proposal_report(&request)?;
    write_text_or_json(options.format, &report, nns_proposal_report_text)
}
