//! Runtime dispatch for direct NNS Governance reports.

use super::{
    commands::{
        governance_command, governance_economics_command, governance_economics_usage_for_error,
        governance_maturity_modulation_command, governance_maturity_modulation_usage_for_error,
        governance_metrics_command, governance_metrics_usage_for_error,
        governance_reward_event_command, governance_reward_event_usage_for_error,
        governance_usage_for_error,
    },
    options::NnsGovernanceOptions,
};
use crate::nns::{
    NnsCommandError, command_args, now_unix_secs, parse_nns_required_subcommand, write_text_or_json,
};
use clap::Command as ClapCommand;
use ic_query::nns::{
    NnsSourceRequest,
    governance::{
        NnsGovernanceHostError, build_nns_governance_economics_report,
        build_nns_governance_maturity_modulation_report, build_nns_governance_metrics_report,
        build_nns_governance_reward_event_report, nns_governance_economics_report_text,
        nns_governance_maturity_modulation_report_text, nns_governance_metrics_report_text,
        nns_governance_reward_event_report_text,
    },
};
use serde::Serialize;
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, governance_usage_for_error) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(governance_command(), args)?;
    match command.as_str() {
        "economics" => run_report(
            args,
            governance_economics_command(),
            governance_economics_usage_for_error,
            build_nns_governance_economics_report,
            nns_governance_economics_report_text,
        ),
        "metrics" => run_report(
            args,
            governance_metrics_command(),
            governance_metrics_usage_for_error,
            build_nns_governance_metrics_report,
            nns_governance_metrics_report_text,
        ),
        "reward-event" => run_report(
            args,
            governance_reward_event_command(),
            governance_reward_event_usage_for_error,
            build_nns_governance_reward_event_report,
            nns_governance_reward_event_report_text,
        ),
        "maturity-modulation" => run_report(
            args,
            governance_maturity_modulation_command(),
            governance_maturity_modulation_usage_for_error,
            build_nns_governance_maturity_modulation_report,
            nns_governance_maturity_modulation_report_text,
        ),
        _ => unreachable!("nns governance dispatch only defines known commands"),
    }
}

fn run_report<I, Report>(
    args: I,
    command: ClapCommand,
    usage: fn() -> String,
    build: fn(&NnsSourceRequest) -> Result<Report, NnsGovernanceHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Report: Serialize,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let options = NnsGovernanceOptions::parse(args, command)?;
    let request = NnsSourceRequest::from_unix_secs(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        "ic-query",
    );
    let report = build(&request)?;
    write_text_or_json(options.format, &report, render_text)
}
