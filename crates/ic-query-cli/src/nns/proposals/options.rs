//! Module: nns::proposals::options
//!
//! Responsibility: parse NNS proposal command options.
//! Does not own: clap command construction, report requests, or live queries.
//! Boundary: converts clap matches into command-local option structs.

use crate::{
    cli::{
        clap::{required_typed, string_option, typed_option},
        common::{output_format, resolve_proposal_sort_direction},
    },
    nns::{
        NnsCommandError, OutputFormat,
        leaf::NnsCommonOptions,
        proposals::values::{
            NNS_PROPOSAL_BALLOTS_FLAG, NNS_PROPOSAL_ID_ARG, NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
            NNS_PROPOSAL_VERBOSE_FLAG, NnsProposalListSortArg, NnsProposalRewardStatusArg,
            NnsProposalStatusArg, NnsProposalTopicArg,
        },
    },
};
use clap::ArgMatches;
use ic_query::nns::proposals::{
    NnsProposalActivityRequest, NnsProposalListSort, NnsProposalRewardStatusFilter,
    NnsProposalSortDirection, NnsProposalStatusFilter, NnsProposalTopicFilter,
};

///
/// NnsProposalActivityOptions
///
/// Output and time-window options for local proposal activity.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalActivityOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) request: NnsProposalActivityRequest,
}

impl NnsProposalActivityOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            request: NnsProposalActivityRequest {
                from_proposal_timestamp_seconds: typed_option(matches, "from"),
                until_proposal_timestamp_seconds: typed_option(matches, "until"),
            },
        }
    }
}

///
/// NnsProposalListOptions
///
/// Options accepted by `icq nns proposal list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalListOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) limit: u32,
    pub(in crate::nns) before_proposal_id: Option<u64>,
    pub(in crate::nns) status: NnsProposalStatusFilter,
    pub(in crate::nns) reward_status: NnsProposalRewardStatusFilter,
    pub(in crate::nns) topic: NnsProposalTopicFilter,
    pub(in crate::nns) proposer_neuron_id: Option<u64>,
    pub(in crate::nns) query: Option<String>,
    pub(in crate::nns) sort: NnsProposalListSort,
    pub(in crate::nns) sort_direction: NnsProposalSortDirection,
    pub(in crate::nns) verbose: bool,
}

impl NnsProposalListOptions {
    pub(in crate::nns) fn from_matches(
        matches: &ArgMatches,
        network: &str,
    ) -> Result<Self, NnsCommandError> {
        let common = NnsCommonOptions::from_matches(matches, network);
        let sort =
            NnsProposalListSort::from(required_typed::<NnsProposalListSortArg>(matches, "sort"));
        let sort_direction = resolve_proposal_sort_direction(
            matches,
            sort.uses_local_direction(),
            sort.default_direction(),
            (
                NnsProposalSortDirection::Asc,
                NnsProposalSortDirection::Desc,
            ),
            || crate::cli::clap::value_names(NnsProposalListSortArg::Api),
        )
        .map_err(NnsCommandError::Usage)?;
        Ok(Self {
            common,
            limit: required_typed(matches, "limit"),
            before_proposal_id: typed_option(matches, "before"),
            status: required_typed::<NnsProposalStatusArg>(matches, "status").into(),
            reward_status: required_typed::<NnsProposalRewardStatusArg>(
                matches,
                NNS_PROPOSAL_LIST_REWARD_STATUS_ARG,
            )
            .into(),
            topic: required_typed::<NnsProposalTopicArg>(matches, "topic").into(),
            proposer_neuron_id: typed_option(matches, "proposer"),
            query: string_option(matches, "query"),
            sort,
            sort_direction,
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        })
    }
}

///
/// NnsProposalOptions
///
/// Options accepted by `icq nns proposal info`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) proposal_id: u64,
    pub(in crate::nns) show_ballots: bool,
    pub(in crate::nns) verbose: bool,
}

impl NnsProposalOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,
            proposal_id: required_typed(matches, NNS_PROPOSAL_ID_ARG),
            show_ballots: matches.get_flag(NNS_PROPOSAL_BALLOTS_FLAG),
            verbose: matches.get_flag(NNS_PROPOSAL_VERBOSE_FLAG),
        }
    }
}
