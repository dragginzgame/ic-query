use super::common::command_unix_secs;
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
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
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_sns_proposal<I>(args: I) -> Result<(), SnsCommandError>
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

pub(super) fn run_sns_proposals<I>(args: I) -> Result<(), SnsCommandError>
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
