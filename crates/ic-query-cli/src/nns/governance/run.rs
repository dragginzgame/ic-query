//! Runtime dispatch for direct NNS Governance reports.

use super::{commands::governance_command, options::NnsGovernanceOptions};
use crate::nns::{NnsCommandError, now_unix_secs, write_text_or_json};
use clap::ArgMatches;
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
pub(in crate::nns) fn command() -> clap::Command {
    governance_command()
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("economics", matches)) => run_report(
            matches,
            network,
            build_nns_governance_economics_report,
            nns_governance_economics_report_text,
        ),
        Some(("metrics", matches)) => run_report(
            matches,
            network,
            build_nns_governance_metrics_report,
            nns_governance_metrics_report_text,
        ),
        Some(("reward-event", matches)) => run_report(
            matches,
            network,
            build_nns_governance_reward_event_report,
            nns_governance_reward_event_report_text,
        ),
        Some(("maturity-modulation", matches)) => run_report(
            matches,
            network,
            build_nns_governance_maturity_modulation_report,
            nns_governance_maturity_modulation_report_text,
        ),
        _ => unreachable!("clap requires a known NNS governance subcommand"),
    }
}

fn run_report<Report>(
    matches: &ArgMatches,
    network: &str,
    build: fn(&NnsSourceRequest) -> Result<Report, NnsGovernanceHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Report: Serialize,
{
    let options = NnsGovernanceOptions::from_matches(matches, network);
    let request = NnsSourceRequest::from_unix_secs(
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        "ic-query",
    );
    let report = build(&request)?;
    write_text_or_json(options.format, &report, render_text)
}
