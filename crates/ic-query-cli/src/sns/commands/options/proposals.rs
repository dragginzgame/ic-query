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
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    let mut request = SnsProposalsRequest::new(lookup, required_typed(matches, "limit"))
        .with_cache_root(cache_root()?)
        .with_status(status)?
        .with_topic(required_typed::<SnsProposalTopicFilter>(matches, "topic"))
        .with_eligibility(required_typed::<SnsProposalEligibilityFilter>(
            matches, "eligible",
        ))
        .with_sort(sort)
        .with_sort_direction(sort_direction);
    request.before_proposal_id = typed_option(matches, "before");
    request.proposer_neuron_id = string_option(matches, "proposer");
    request.query = string_option(matches, "query");
    Ok((format, request))
}

pub(in crate::sns::commands) fn proposal_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsProposalRequest), SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    let cache_root = lookup
        .root_canister_id()
        .map(|_| cache_root())
        .transpose()?;
    Ok((
        format,
        SnsProposalRequest {
            lookup,
            proposal_id: required_typed(matches, "proposal-id"),
            cache_root,
        },
    ))
}
