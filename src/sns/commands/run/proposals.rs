use super::common::{command_args, lookup_command_parts};
use crate::{
    cli::common::write_text_or_json,
    sns::{
        commands::{
            SnsCommandError,
            options::{SnsProposalOptions, SnsProposalsOptions},
            spec::{sns_proposal_usage, sns_proposals_usage},
        },
        report::{
            SnsProposalRequest, SnsProposalsRequest, build_sns_proposal_report,
            build_sns_proposals_report, sns_proposal_report_text, sns_proposals_report_text,
        },
    },
};
use std::ffi::OsString;

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
        verbose: options.verbose,
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
        verbose: options.verbose,
    };
    let report = build_sns_proposals_report(&request)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}
