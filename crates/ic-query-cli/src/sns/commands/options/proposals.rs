//! Module: sns::commands::options::proposals
//!
//! Responsibility: parse SNS proposal detail, list, refresh, and cache options.
//! Does not own: proposal command specs, proposal cache policy, or reports.
//! Boundary: validates clap matches into proposal command request inputs.

use crate::{
    cli::clap::{required_typed, string_option, typed_option},
    sns::commands::{
        SnsCommandError,
        options::lookup::SnsLookupOptions,
        spec::{
            SnsProposalEligibilityArg, SnsProposalStatusArg, SnsProposalTopicArg,
            SnsProposalsSortArg,
        },
    },
};
use clap::ArgMatches;
use ic_query::sns::{SnsProposalSortDirection, SnsProposalsSort};

///
/// SnsProposalsOptions
///
/// Parsed options accepted by `icq sns proposal list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) limit: u32,
    pub(in crate::sns::commands) before_proposal_id: Option<u64>,
    pub(in crate::sns::commands) status: SnsProposalStatusArg,
    pub(in crate::sns::commands) topic: SnsProposalTopicArg,
    pub(in crate::sns::commands) eligibility: SnsProposalEligibilityArg,
    pub(in crate::sns::commands) proposer_neuron_id: Option<String>,
    pub(in crate::sns::commands) query: Option<String>,
    pub(in crate::sns::commands) sort: SnsProposalsSortArg,
    pub(in crate::sns::commands) sort_direction: SnsProposalSortDirection,
    pub(in crate::sns::commands) verbose: bool,
}

///
/// SnsProposalOptions
///
/// Parsed options accepted by `icq sns proposal info`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsProposalOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) proposal_id: u64,
    pub(in crate::sns::commands) verbose: bool,
    pub(in crate::sns::commands) show_ballots: bool,
}

impl SnsProposalsOptions {
    pub(in crate::sns::commands) fn from_matches(
        matches: &ArgMatches,
        network: &str,
    ) -> Result<Self, SnsCommandError> {
        let status = required_typed(matches, "status");
        let sort = required_typed(matches, "sort");
        let sort_direction = proposal_sort_direction(matches, sort)?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            limit: required_typed(matches, "limit"),
            before_proposal_id: typed_option::<u64>(matches, "before"),
            status,
            topic: required_typed(matches, "topic"),
            eligibility: required_typed(matches, "eligible"),
            proposer_neuron_id: string_option(matches, "proposer"),
            query: string_option(matches, "query"),
            sort,
            sort_direction,
            verbose: matches.get_flag("verbose"),
        })
    }
}

fn proposal_sort_direction(
    matches: &ArgMatches,
    sort: SnsProposalsSortArg,
) -> Result<SnsProposalSortDirection, SnsCommandError> {
    if matches.get_flag("asc") {
        return explicit_proposal_sort_direction(sort, SnsProposalSortDirection::Asc, "--asc");
    }
    if matches.get_flag("desc") {
        return explicit_proposal_sort_direction(sort, SnsProposalSortDirection::Desc, "--desc");
    }
    Ok(SnsProposalsSort::from(sort).default_direction())
}

fn explicit_proposal_sort_direction(
    sort: SnsProposalsSortArg,
    direction: SnsProposalSortDirection,
    flag: &'static str,
) -> Result<SnsProposalSortDirection, SnsCommandError> {
    if !SnsProposalsSort::from(sort).uses_local_direction() {
        return Err(SnsCommandError::Usage(format!(
            "{flag} requires --sort {}",
            crate::cli::clap::value_names(SnsProposalsSortArg::Api)
        )));
    }
    Ok(direction)
}

impl SnsProposalOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            proposal_id: required_typed(matches, "proposal-id"),
            verbose: matches.get_flag("verbose"),
            show_ballots: matches.get_flag("ballots"),
        }
    }
}
