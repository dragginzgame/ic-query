//! Module: sns::commands::options::proposals
//!
//! Responsibility: parse SNS proposal detail, list, refresh, and cache options.
//! Does not own: proposal command specs, proposal cache policy, or reports.
//! Boundary: validates clap matches into proposal command request inputs.

use crate::{
    cli::clap::{required_typed, string_option, typed_option},
    cli::common::resolve_proposal_sort_direction,
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
use ic_query::sns::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};

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
    pub(in crate::sns::commands) status: SnsProposalStatusFilter,
    pub(in crate::sns::commands) topic: SnsProposalTopicFilter,
    pub(in crate::sns::commands) eligibility: SnsProposalEligibilityFilter,
    pub(in crate::sns::commands) proposer_neuron_id: Option<String>,
    pub(in crate::sns::commands) query: Option<String>,
    pub(in crate::sns::commands) sort: SnsProposalsSort,
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
        let status = required_typed::<SnsProposalStatusArg>(matches, "status").into();
        let sort = SnsProposalsSort::from(required_typed::<SnsProposalsSortArg>(matches, "sort"));
        let sort_direction = resolve_proposal_sort_direction(
            matches,
            sort.uses_local_direction(),
            sort.default_direction(),
            (
                SnsProposalSortDirection::Asc,
                SnsProposalSortDirection::Desc,
            ),
            || crate::cli::clap::value_names(SnsProposalsSortArg::Api),
        )
        .map_err(SnsCommandError::Usage)?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            limit: required_typed(matches, "limit"),
            before_proposal_id: typed_option::<u64>(matches, "before"),
            status,
            topic: required_typed::<SnsProposalTopicArg>(matches, "topic").into(),
            eligibility: required_typed::<SnsProposalEligibilityArg>(matches, "eligible").into(),
            proposer_neuron_id: string_option(matches, "proposer"),
            query: string_option(matches, "query"),
            sort,
            sort_direction,
            verbose: matches.get_flag("verbose"),
        })
    }
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
