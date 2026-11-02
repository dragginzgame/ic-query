//! Module: sns::commands::options::proposals
//!
//! Responsibility: parse SNS proposal detail, list, refresh, and cache options.
//! Does not own: proposal command specs, proposal cache policy, or reports.
//! Boundary: validates clap matches into proposal command request inputs.

use crate::cli::common::OutputFormat;
use crate::{
    cli::clap::{required_typed, string_option, typed_option},
    cli::common::resolve_proposal_sort_direction,
    sns::commands::{SnsCommandError, options::lookup::lookup_request},
};
use clap::ArgMatches;
use ic_query::sns::SnsProposalEligibilityFilter;
use ic_query::sns::SnsProposalRequest;
use ic_query::sns::SnsProposalStatusFilter;
use ic_query::sns::SnsProposalTopicFilter;
use ic_query::sns::SnsProposalsRequest;
use ic_query::sns::SnsProposalsSort;
use std::path::PathBuf;

pub(in crate::sns::commands) fn proposals_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsProposalsRequest), SnsCommandError> {
    let status = required_typed::<SnsProposalStatusFilter>(matches, "status");
    let sort = required_typed::<SnsProposalsSort>(matches, "sort");
    let sort_direction = resolve_proposal_sort_direction(
        matches,
        sort.uses_local_direction(),
        sort.default_direction(),
        || {
            SnsProposalsSort::ALL
                .iter()
                .copied()
                .filter(|sort| sort.uses_local_direction())
                .map(SnsProposalsSort::as_str)
                .collect::<Vec<_>>()
                .join("|")
        },
    )
    .map_err(SnsCommandError::Usage)?;
    let (format, lookup) = lookup_request(matches, network, now_unix_secs);
    Ok((
        format,
        SnsProposalsRequest {
            lookup,
            limit: required_typed(matches, "limit"),
            before_proposal_id: typed_option(matches, "before"),
            status,
            topic: required_typed::<SnsProposalTopicFilter>(matches, "topic"),
            eligibility: required_typed::<SnsProposalEligibilityFilter>(matches, "eligible"),
            proposer_neuron_id: string_option(matches, "proposer"),
            query: string_option(matches, "query"),
            sort,
            sort_direction,
            cache_root: Some(cache_root()?),
            verbose: matches.get_flag("verbose"),
        },
    ))
}

pub(in crate::sns::commands) fn proposal_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsProposalRequest), SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs);
    let cache_root = ic_platform::candid::contract::Principal::from_text(&lookup.input)
        .ok()
        .map(|_| cache_root())
        .transpose()?;
    Ok((
        format,
        SnsProposalRequest {
            lookup,
            proposal_id: required_typed(matches, "proposal-id"),
            cache_root,
            verbose: matches.get_flag("verbose"),
            show_ballots: matches.get_flag("ballots"),
        },
    ))
}
